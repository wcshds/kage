use crate::{font::ming::Ming, line::stroke_line::StrokeLineType, two_d, utils::Point};

#[derive(Debug, Clone, Copy)]
pub struct AdjustedStroke {
    /// origin name: kirikuchiAdjustment
    pub(crate) slash_adjustment: f64,
    /// origin name: tate adjustment
    pub(crate) vertical_adjustment: f64,
    /// origin name: hane adjustment
    pub(crate) flick_adjustment: f64,
    /// origin name: uroko adjustment
    pub(crate) triangle_adjustment: usize,
    /// origin name: kakato adjustment
    pub(crate) foot_adjustment: usize,
    /// origin name: mage adjustment
    pub(crate) curve_adjustment: f64,
}

pub trait StrokeAdjustmentTrait {
    fn adjust_strokes<'a>(
        &self,
        line_type: &'a [&'a StrokeLineType],
    ) -> Vec<(&'a StrokeLineType, AdjustedStroke)>;
}

impl StrokeAdjustmentTrait for Ming {
    fn adjust_strokes<'a>(
        &self,
        line_type: &'a [&'a StrokeLineType],
    ) -> Vec<(&'a StrokeLineType, AdjustedStroke)> {
        #[inline]
        fn for_each_segment<F: FnMut(Point, Point)>(stroke: &StrokeLineType, mut f: F) {
            if stroke.stroke_type.opt != 0 {
                f(stroke.point_1, stroke.point_2);
                return;
            }

            match stroke.stroke_type.kind {
                crate::line::stroke_line::StrokeKind::StraightLine => {
                    f(stroke.point_1, stroke.point_2);
                }
                crate::line::stroke_line::StrokeKind::Curve
                | crate::line::stroke_line::StrokeKind::BendLine
                | crate::line::stroke_line::StrokeKind::OtsuCurve => {
                    f(stroke.point_1, stroke.point_2);
                    f(stroke.point_2, stroke.point_3);
                }
                crate::line::stroke_line::StrokeKind::ComplexCurve
                | crate::line::stroke_line::StrokeKind::VerticalSlash => {
                    f(stroke.point_1, stroke.point_2);
                    f(stroke.point_2, stroke.point_3);
                    f(stroke.point_3, stroke.point_4);
                }
                crate::line::stroke_line::StrokeKind::Unknown => {}
            }
        }

        #[inline]
        fn crosses(stroke: &StrokeLineType, start: Point, end: Point) -> bool {
            let mut hit = false;
            for_each_segment(stroke, |s1, s2| {
                if !hit && two_d::is_cross(s1, s2, start, end) {
                    hit = true;
                }
            });
            hit
        }

        #[inline]
        fn crosses_box(stroke: &StrokeLineType, diag_1: Point, diag_2: Point) -> bool {
            let mut hit = false;
            for_each_segment(stroke, |s1, s2| {
                if !hit && two_d::is_cross_box(s1, s2, diag_1, diag_2) {
                    hit = true;
                }
            });
            hit
        }

        #[inline]
        fn unit_vector(dx: f64, dy: f64) -> (f64, f64) {
            let len_sq = dx * dx + dy * dy;
            if len_sq == 0.0 {
                return (0.0, 0.0);
            }
            let len = len_sq.sqrt();
            (dx / len, dy / len)
        }

        let mut adjusted: Vec<(&StrokeLineType, AdjustedStroke)> =
            Vec::with_capacity(line_type.len());

        for &stroke in line_type {
            adjusted.push((
                stroke,
                AdjustedStroke {
                    slash_adjustment: stroke.head_shape.opt_1 as f64,
                    vertical_adjustment: stroke.head_shape.opt_2 as f64
                        + stroke.head_shape.opt_3 as f64 * 10.0,
                    flick_adjustment: stroke.tail_shape.opt_1 as f64,
                    triangle_adjustment: stroke.tail_shape.opt as usize,
                    foot_adjustment: stroke.tail_shape.opt as usize,
                    curve_adjustment: stroke.tail_shape.opt_2 as f64,
                },
            ));
        }

        // adjust hane
        {
            let mut vert_segments = Vec::new();
            for (idx, (stroke, _)) in adjusted.iter().enumerate() {
                if stroke.stroke_type.base == 1
                    && stroke.stroke_type.opt == 0
                    && stroke.point_1.x == stroke.point_2.x
                {
                    vert_segments.push((idx, stroke.point_1.x, stroke.point_1.y, stroke.point_2.y));
                }
            }

            for (idx, (stroke, adj)) in adjusted.iter_mut().enumerate() {
                let base = stroke.stroke_type.base;
                if (base == 1 || base == 2 || base == 6)
                    && stroke.stroke_type.opt == 0
                    && stroke.tail_shape.base == 4
                    && stroke.tail_shape.opt == 0
                {
                    let (lpx, lpy) = match base {
                        1 => (stroke.point_2.x, stroke.point_2.y),
                        2 => (stroke.point_3.x, stroke.point_3.y),
                        _ => (stroke.point_4.x, stroke.point_4.y),
                    };

                    let mut nearest = f64::INFINITY;
                    if lpx + 18.0 < 100.0 {
                        nearest = lpx + 18.0;
                    }

                    for &(other_idx, x, y1, y2) in &vert_segments {
                        if idx != other_idx && lpx - x < 100.0 && x < lpx && y1 <= lpy && y2 >= lpy
                        {
                            let diff = lpx - x;
                            if diff < nearest {
                                nearest = diff;
                            }
                        }
                    }

                    if nearest.is_finite() {
                        adj.flick_adjustment += 7.0 - (nearest / 15.0).floor();
                    }
                }
            }
        }

        // adjust mage
        {
            let mut hori_segments = Vec::new();
            for (idx, (stroke, _)) in adjusted.iter().enumerate() {
                if stroke.stroke_type.base == 1
                    && stroke.stroke_type.opt == 0
                    && stroke.point_1.y == stroke.point_2.y
                {
                    hori_segments.push((
                        idx,
                        false,
                        stroke.point_2.y,
                        stroke.point_1.x,
                        stroke.point_2.x,
                    ));
                } else if stroke.stroke_type.base == 3
                    && stroke.stroke_type.opt == 0
                    && stroke.point_2.y == stroke.point_3.y
                {
                    hori_segments.push((
                        idx,
                        true,
                        stroke.point_2.y,
                        stroke.point_2.x,
                        stroke.point_3.x,
                    ));
                }
            }

            for &(target_idx, is_target, y, x1, x2) in &hori_segments {
                if is_target {
                    for &(other_idx, _, other_y, other_x1, other_x2) in &hori_segments {
                        if target_idx != other_idx && !(x1 + 1.0 > other_x2 || x2 - 1.0 < other_x1)
                        {
                            let dy = (y - other_y).abs();
                            if dy.round() < self.min_width_vertical * self.k_adjust_curve_step {
                                let adj = &mut adjusted[target_idx].1;
                                adj.curve_adjustment += self.k_adjust_curve_step
                                    - (dy / self.min_width_vertical).floor();
                                if adj.curve_adjustment > self.k_adjust_curve_step {
                                    adj.curve_adjustment = self.k_adjust_curve_step;
                                }
                            }
                        }
                    }
                }
            }
        }

        // adjust tate
        {
            let mut vert_segments = Vec::new();
            for (idx, (stroke, _)) in adjusted.iter().enumerate() {
                if (stroke.stroke_type.base == 1
                    || stroke.stroke_type.base == 3
                    || stroke.stroke_type.base == 7)
                    && stroke.stroke_type.opt == 0
                    && stroke.point_1.x == stroke.point_2.x
                {
                    vert_segments.push((idx, stroke.point_1.x, stroke.point_1.y, stroke.point_2.y));
                }
            }

            for &(idx, x, y1, y2) in &vert_segments {
                let head_shape = adjusted[idx].0.head_shape;
                for &(other_idx, other_x, other_y1, other_y2) in &vert_segments {
                    if idx != other_idx && !(y1 + 1.0 > other_y2 || y2 - 1.0 < other_y1) {
                        let dx = (x - other_x).abs();
                        if dx.round() < self.min_width_vertical * self.k_adjust_vertical_step {
                            let adj = &mut adjusted[idx].1;
                            adj.vertical_adjustment +=
                                self.k_adjust_vertical_step - (dx / self.min_width_vertical).floor();
                            if adj.vertical_adjustment > self.k_adjust_vertical_step
                                || (adj.vertical_adjustment == self.k_adjust_vertical_step
                                    && (head_shape.opt_1 != 0 || head_shape.base != 0))
                            {
                                adj.vertical_adjustment = self.k_adjust_vertical_step;
                            }
                        }
                    }
                }
            }
        }

        // adjust kakato
        {
            let step = self.k_adjust_foot_step as usize;
            for idx in 0..adjusted.len() {
                let stroke = adjusted[idx].0;
                if stroke.stroke_type.base == 1
                    && stroke.stroke_type.opt == 0
                    && (stroke.tail_shape.base == 13 || stroke.tail_shape.base == 23)
                    && stroke.tail_shape.opt == 0
                {
                    let mut foot = None;

                    for k in 0..step {
                        let y_range_next = self.k_adjust_foot_range_y[k + 1];
                        let collide = (0..adjusted.len()).any(|other_idx| {
                            if idx == other_idx {
                                return false;
                            }
                            let stroke2 = adjusted[other_idx].0;
                            crosses_box(
                                stroke2,
                                (
                                    stroke.point_2.x - self.k_adjust_foot_range_x / 2.0,
                                    stroke.point_2.y + self.k_adjust_foot_range_y[k],
                                    None,
                                )
                                    .into(),
                                (
                                    stroke.point_2.x + self.k_adjust_foot_range_x / 2.0,
                                    stroke.point_2.y + y_range_next,
                                    None,
                                )
                                    .into(),
                            )
                        });

                        if collide
                            || (stroke.point_2.y + y_range_next).round() > 200.0
                            || (stroke.point_2.y - stroke.point_1.y).round() < y_range_next
                        {
                            if 3 >= k {
                                foot = Some(3 - k);
                            }
                            break;
                        }
                    }

                    if let Some(value) = foot {
                        adjusted[idx].1.foot_adjustment = value;
                    }
                }
            }
        }

        // adjust uroko
        {
            let length_steps = self.k_adjust_triangle_length_step as usize;
            for idx in 0..adjusted.len() {
                let stroke = adjusted[idx].0;
                if stroke.stroke_type.base == 1
                    && stroke.stroke_type.opt == 0
                    && stroke.tail_shape.base == 0
                    && stroke.tail_shape.opt == 0
                {
                    let mut new_tri = adjusted[idx].1.triangle_adjustment;
                    for k in 0..length_steps {
                        let (cosrad, sinrad) = if stroke.point_1.y == stroke.point_2.y {
                            (1.0, 0.0)
                        } else if stroke.point_2.x - stroke.point_1.x < 0.0 {
                            unit_vector(
                                stroke.point_1.x - stroke.point_2.x,
                                stroke.point_1.y - stroke.point_2.y,
                            )
                        } else {
                            unit_vector(
                                stroke.point_2.x - stroke.point_1.x,
                                stroke.point_2.y - stroke.point_1.y,
                            )
                        };

                        let tx =
                            stroke.point_2.x - self.k_adjust_triangle_line[k] * cosrad - 0.5 * sinrad;
                        let ty =
                            stroke.point_2.y - self.k_adjust_triangle_line[k] * sinrad - 0.5 * cosrad;

                        let tlen = if stroke.point_1.y == stroke.point_2.y {
                            stroke.point_2.x - stroke.point_1.x
                        } else {
                            (stroke.point_2.y - stroke.point_1.y)
                                .hypot(stroke.point_2.x - stroke.point_1.x)
                        };

                        let hit = (0..adjusted.len()).any(|other_idx| {
                            idx != other_idx
                                && crosses(
                                    adjusted[other_idx].0,
                                    (tx, ty, None).into(),
                                    stroke.point_2,
                                )
                        });

                        if tlen.round() < self.k_adjust_triangle_length[k] || hit {
                            new_tri = self.k_adjust_triangle_length_step as usize - k;
                            break;
                        }
                    }
                    adjusted[idx].1.triangle_adjustment = new_tri;
                }
            }
        }

        // adjust uroko2
        {
            let mut hori_segments = Vec::new();
            for (idx, (stroke, adj)) in adjusted.iter().enumerate() {
                if stroke.stroke_type.base == 1
                    && stroke.stroke_type.opt == 0
                    && stroke.point_1.y == stroke.point_2.y
                {
                    let is_target = stroke.tail_shape.base == 0
                        && stroke.tail_shape.opt == 0
                        && adj.triangle_adjustment == 0;
                    hori_segments.push((
                        idx,
                        is_target,
                        stroke.point_1.y,
                        stroke.point_1.x,
                        stroke.point_2.x,
                    ));
                } else if stroke.stroke_type.base == 3
                    && stroke.stroke_type.opt == 0
                    && stroke.point_2.y == stroke.point_3.y
                {
                    hori_segments.push((
                        idx,
                        false,
                        stroke.point_2.y,
                        stroke.point_2.x,
                        stroke.point_3.x,
                    ));
                }
            }

            for &(idx, is_target, y, x1, x2) in &hori_segments {
                if is_target {
                    let mut pressure = 0.0;
                    for &(other_idx, _, other_y, other_x1, other_x2) in &hori_segments {
                        if idx != other_idx && !(x1 + 1.0 > other_x2 || x2 - 1.0 < other_x1) {
                            let dy = (y - other_y).abs();
                            if dy.round() < self.k_adjust_triangle2_length {
                                let delta = self.k_adjust_triangle2_length - dy;
                                pressure += delta.powf(1.1);
                            }
                        }
                    }

                    let value = (pressure / self.k_adjust_triangle2_length).floor();
                    let capped = value.min(self.k_adjust_triangle2_step) as usize;
                    adjusted[idx].1.triangle_adjustment = capped;
                }
            }
        }

        // adjust kirikuchi
        {
            let mut hori_segments = Vec::new();
            for (stroke, _) in &adjusted {
                if stroke.stroke_type.base == 1
                    && stroke.stroke_type.opt == 0
                    && stroke.point_1.y == stroke.point_2.y
                {
                    hori_segments.push((stroke.point_1.y, stroke.point_1.x, stroke.point_2.x));
                }
            }

            for (stroke, adj) in adjusted.iter_mut() {
                if stroke.stroke_type.base == 2
                    && stroke.stroke_type.opt == 0
                    && stroke.head_shape.base == 32
                    && stroke.head_shape.opt == 0
                    && stroke.point_1.x > stroke.point_2.x
                    && stroke.point_1.y < stroke.point_2.y
                {
                    let hit = hori_segments.iter().any(|&(y, x1, x2)| {
                        x1 < stroke.point_1.x && x2 > stroke.point_1.x && y == stroke.point_1.y
                    });
                    if hit {
                        adj.slash_adjustment = 1.0;
                    }
                }
            }
        }

        adjusted
    }
}

#[cfg(test)]
mod test {}
