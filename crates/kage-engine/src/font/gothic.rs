use crate::{
    curve::{FattenResult, generate_fatten_curve},
    line::stroke_line::{EndKind, EndType, StrokeKind, StrokeLineType},
    pen::Pen,
    polygon::Polygon,
    polygons::Polygons,
    utils::{Point, Rgb, Vector, normalize},
};

pub struct Gothic {
    /// must divide 1000
    pub sample_step: usize,
    pub width: f64,
    /// Size of kakato in gothic.
    pub foot_size: f64,
    pub curve_size: f64,
}

impl Gothic {
    pub fn new() -> Self {
        Self {
            sample_step: 100,
            width: 5.0,
            foot_size: 3.0,
            curve_size: 10.0,
        }
    }
}

impl Gothic {
    fn draw_curve_body<P1, P2, P3, P4>(
        &self,
        polygons: &mut Polygons,
        start_point: P1,
        control_point_1: P2,
        control_point_2: P3,
        end_point: P4,
        color: Option<Rgb>,
    ) where
        P1: Into<Point>,
        P2: Into<Point>,
        P3: Into<Point>,
        P4: Into<Point>,
    {
        let start_point = start_point.into();
        let control_point_1 = control_point_1.into();
        let control_point_2 = control_point_2.into();
        let end_point = end_point.into();

        let FattenResult {
            left: left_sampled_points,
            right: right_sampled_points,
        } = generate_fatten_curve(
            start_point,
            control_point_1,
            control_point_2,
            end_point,
            self.sample_step,
            |_| self.width,
        );

        let mut polygon_1 = Polygon::new(left_sampled_points, color);
        let mut polygon_2 = Polygon::new(right_sampled_points, color);

        polygon_2.reverse();
        polygon_1.concat(polygon_2);
        polygons.push(polygon_1);
    }

    fn cd_draw_curve_universal<P1, P2, P3, P4>(
        &self,
        polygons: &mut Polygons,
        start_point: P1,
        control_point_1: P2,
        control_point_2: P3,
        end_point: P4,
        head_shape: EndType,
        tail_shape: EndType,
        color: Option<Rgb>,
    ) where
        P1: Into<Point>,
        P2: Into<Point>,
        P3: Into<Point>,
        P4: Into<Point>,
    {
        let mut start_point = start_point.into();
        let control_point_1 = control_point_1.into();
        let control_point_2 = control_point_2.into();
        let mut end_point = end_point.into();

        let delta_1 = match &head_shape.kind {
            &EndKind::HorizontalConnection
            | &EndKind::VerticalConnection
            | &EndKind::TopLeftCorner
            | &EndKind::TopRightCorner => self.width,
            &EndKind::BottomLeftCorner
            | &EndKind::BottomRightCorner
            | &EndKind::BottomLeftZhOld
            | &EndKind::BottomLeftZhNew => self.width * self.foot_size,
            _ => 0.0,
        };

        if delta_1 != 0.0 {
            let delta_vector =
                if start_point.x == control_point_1.x && start_point.y == control_point_1.y {
                    (0.0, delta_1).into()
                } else {
                    normalize(start_point - control_point_1, delta_1)
                };
            start_point = start_point + delta_vector;
        }

        let delta_2 = match &tail_shape.kind {
            &EndKind::HorizontalConnection
            | &EndKind::VerticalConnection
            | &EndKind::TopLeftCorner
            | &EndKind::TopRightCorner => self.width,
            &EndKind::BottomLeftCorner
            | &EndKind::BottomRightCorner
            | &EndKind::BottomLeftZhOld
            | &EndKind::BottomLeftZhNew => self.width * self.foot_size,
            _ => 0.0,
        };

        if delta_2 != 0.0 {
            let delta_vector =
                if control_point_2.x == end_point.x && control_point_2.y == end_point.y {
                    (0.0, -delta_2).into()
                } else {
                    normalize(end_point - control_point_2, delta_2)
                };
            end_point = end_point + delta_vector;
        }

        self.draw_curve_body(
            polygons,
            start_point,
            control_point_1,
            control_point_2,
            end_point,
            color,
        );
    }

    fn cd_draw_quadratic_bezier<P1, P2, P3>(
        &self,
        polygons: &mut Polygons,
        start_point: P1,
        control_point: P2,
        end_point: P3,
        head_shape: EndType,
        tail_shape: EndType,
        color: Option<Rgb>,
    ) where
        P1: Into<Point>,
        P2: Into<Point>,
        P3: Into<Point>,
    {
        let control_point = control_point.into();

        self.cd_draw_curve_universal(
            polygons,
            start_point,
            control_point,
            control_point,
            end_point,
            head_shape,
            tail_shape,
            color,
        )
    }

    fn cd_draw_cubic_bezier<P1, P2, P3, P4>(
        &self,
        polygons: &mut Polygons,
        start_point: P1,
        control_point_1: P2,
        control_point_2: P3,
        end_point: P4,
        head_shape: EndType,
        tail_shape: EndType,
        color: Option<Rgb>,
    ) where
        P1: Into<Point>,
        P2: Into<Point>,
        P3: Into<Point>,
        P4: Into<Point>,
    {
        self.cd_draw_curve_universal(
            polygons,
            start_point,
            control_point_1,
            control_point_2,
            end_point,
            head_shape,
            tail_shape,
            color,
        )
    }

    fn cd_draw_line<P1, P2>(
        &self,
        polygons: &mut Polygons,
        start_point: P1,
        end_point: P2,
        head_shape: EndType,
        tail_shape: EndType,
        color: Option<Rgb>,
    ) where
        P1: Into<Point>,
        P2: Into<Point>,
    {
        let start_point = start_point.into();
        let end_point = end_point.into();

        let (mut pen_1, mut pen_2, end_shape_1, end_shape_2) = if (start_point.x == end_point.x
            && start_point.y > end_point.y)
            || (start_point.x > end_point.x)
        {
            (
                Pen::new(end_point.x, end_point.y),
                Pen::new(start_point.x, start_point.y),
                tail_shape,
                head_shape,
            )
        } else {
            (
                Pen::new(start_point.x, start_point.y),
                Pen::new(end_point.x, end_point.y),
                head_shape,
                tail_shape,
            )
        };

        // Avoid the degenerate case where the line collapses to a point,
        // since we can't compute a reliable normal vector then.
        if (start_point.x != end_point.x) || (start_point.y != end_point.y) {
            pen_1.set_down(pen_2.global_point.x, pen_2.global_point.y);
            pen_2.set_up(pen_1.global_point.x, pen_1.global_point.y);
        }

        match &end_shape_1.kind {
            &EndKind::HorizontalConnection
            | &EndKind::VerticalConnection
            | &EndKind::TopLeftCorner
            | &EndKind::TopRightCorner => {
                pen_1.move_local(0.0, -self.width);
            }
            &EndKind::BottomLeftCorner
            | &EndKind::BottomRightCorner
            | &EndKind::BottomLeftZhOld
            | &EndKind::BottomLeftZhNew => {
                pen_1.move_local(0.0, -self.width * self.foot_size);
            }
            _ => {}
        }

        match &end_shape_2.kind {
            &EndKind::HorizontalConnection
            | &EndKind::VerticalConnection
            | &EndKind::TopLeftCorner
            | &EndKind::TopRightCorner => {
                pen_2.move_local(0.0, self.width);
            }
            &EndKind::BottomLeftCorner
            | &EndKind::BottomRightCorner
            | &EndKind::BottomLeftZhOld
            | &EndKind::BottomLeftZhNew => {
                pen_2.move_local(0.0, self.width * self.foot_size);
            }
            _ => {}
        }

        let mut polygon = Polygon::new(
            vec![
                pen_1.get_point(self.width, 0.0, false),
                pen_2.get_point(self.width, 0.0, false),
                pen_2.get_point(-self.width, 0.0, false),
                pen_1.get_point(-self.width, 0.0, false),
            ],
            color,
        );

        // The reason for this block is unclear; keeping it causes the downstream
        // path-combining task to produce holes in the SVG.
        if start_point.x == end_point.x {
            polygon.reverse();
        }

        polygons.push(polygon);
    }
}

impl Gothic {
    pub fn df_draw_font(&self, polygons: &mut Polygons, stroke_line: StrokeLineType) {
        match stroke_line.stroke_type.kind {
            StrokeKind::StraightLine => {
                if matches!(&stroke_line.tail_shape.kind, EndKind::LeftUpwardFlick) {
                    let delta_vector = if stroke_line.point_1.x == stroke_line.point_2.x
                        && stroke_line.point_1.y == stroke_line.point_2.y
                    {
                        (0.0, self.curve_size).into()
                    } else {
                        normalize(stroke_line.point_1 - stroke_line.point_2, self.curve_size)
                    };
                    let joint_point = stroke_line.point_2 + delta_vector;
                    self.cd_draw_line(
                        polygons,
                        stroke_line.point_1,
                        joint_point,
                        stroke_line.head_shape,
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        joint_point,
                        stroke_line.point_2,
                        (
                            stroke_line.point_2.x - self.curve_size * 2.0,
                            stroke_line.point_2.y - self.curve_size * 0.5,
                        ),
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else {
                    self.cd_draw_line(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        stroke_line.head_shape,
                        stroke_line.tail_shape,
                        stroke_line.color,
                    );
                }
            }
            StrokeKind::Curve => {
                // There is no 12 for the first column in glyphwiki data, so we only need to keep the logic of `case 2`.
                // TODO: slash_adjustment is too coupled, it is only used in `a2 === 132`
                if matches!(&stroke_line.tail_shape.kind, EndKind::LeftUpwardFlick) {
                    let delta_vector = if stroke_line.point_2.x == stroke_line.point_3.x {
                        (0.0, -self.curve_size).into()
                    } else if stroke_line.point_2.y == stroke_line.point_3.y {
                        (-self.curve_size, 0.0).into()
                    } else {
                        normalize(stroke_line.point_2 - stroke_line.point_3, self.curve_size)
                    };
                    let joint_point = stroke_line.point_3 + delta_vector;

                    self.cd_draw_quadratic_bezier(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        joint_point,
                        stroke_line.head_shape,
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        joint_point,
                        stroke_line.point_3,
                        (
                            stroke_line.point_3.x - self.curve_size * 2.0,
                            stroke_line.point_3.y - self.curve_size * 0.5,
                            false,
                        ),
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else if matches!(&stroke_line.tail_shape.kind, EndKind::RightUpwardFlick)
                    && stroke_line.tail_shape.opt == 0
                {
                    let flick_control_point: Point = (
                        stroke_line.point_3.x + self.curve_size,
                        stroke_line.point_3.y,
                    )
                        .into();
                    let flick_end_point = (
                        flick_control_point.x + self.curve_size * 0.5,
                        stroke_line.point_3.y - self.curve_size * 2.0,
                    );

                    self.cd_draw_quadratic_bezier(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        stroke_line.point_3,
                        stroke_line.head_shape,
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        stroke_line.point_3,
                        flick_control_point,
                        flick_end_point,
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else {
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        stroke_line.point_3,
                        stroke_line.head_shape,
                        stroke_line.tail_shape,
                        stroke_line.color,
                    );
                }
            }
            StrokeKind::BendLine => {
                let delta_vector_1 = if stroke_line.point_1.x == stroke_line.point_2.x
                    && stroke_line.point_1.y == stroke_line.point_2.y
                {
                    (0.0, self.curve_size).into()
                } else {
                    normalize(stroke_line.point_1 - stroke_line.point_2, self.curve_size)
                };
                let joint_point_1 = stroke_line.point_2 + delta_vector_1;

                let delta_vector_2 = if stroke_line.point_2.x == stroke_line.point_3.x
                    && stroke_line.point_2.y == stroke_line.point_3.y
                {
                    (0.0, -self.curve_size).into()
                } else {
                    normalize(stroke_line.point_3 - stroke_line.point_2, self.curve_size)
                };
                let joint_point_2 = stroke_line.point_2 + delta_vector_2;

                self.cd_draw_line(
                    polygons,
                    stroke_line.point_1,
                    joint_point_1,
                    stroke_line.head_shape,
                    EndType::new(1.0),
                    stroke_line.color,
                );
                self.cd_draw_quadratic_bezier(
                    polygons,
                    joint_point_1,
                    stroke_line.point_2,
                    joint_point_2,
                    EndType::new(1.0),
                    EndType::new(1.0),
                    stroke_line.color,
                );

                if matches!(&stroke_line.tail_shape.kind, EndKind::RightUpwardFlick)
                    && stroke_line.tail_shape.opt_1 == 0
                {
                    let joint_point_3 = (
                        stroke_line.point_3.x - self.curve_size,
                        stroke_line.point_3.y,
                    );
                    let flick_end_point = (
                        stroke_line.point_3.x + self.curve_size * 0.5,
                        stroke_line.point_3.y - self.curve_size * 2.0,
                    );

                    self.cd_draw_line(
                        polygons,
                        joint_point_2,
                        joint_point_3,
                        EndType::new(1.0),
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        joint_point_3,
                        stroke_line.point_3,
                        flick_end_point,
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else {
                    self.cd_draw_line(
                        polygons,
                        joint_point_2,
                        stroke_line.point_3,
                        EndType::new(1.0),
                        stroke_line.tail_shape,
                        stroke_line.color,
                    );
                }
            }
            StrokeKind::OtsuCurve => {
                let scale_factor =
                    (Vector::from(stroke_line.point_3 - stroke_line.point_2).hypot() / 120.0 * 6.0)
                        .min(6.0);

                let delta_vector_1 = if stroke_line.point_1.x == stroke_line.point_2.x
                    && stroke_line.point_1.y == stroke_line.point_2.y
                {
                    (0.0, self.curve_size * scale_factor).into()
                } else {
                    normalize(
                        stroke_line.point_1 - stroke_line.point_2,
                        self.curve_size * scale_factor,
                    )
                };
                let joint_point_1 = stroke_line.point_2 + delta_vector_1;

                let delta_vector_2 = if stroke_line.point_2.x == stroke_line.point_3.x
                    && stroke_line.point_2.y == stroke_line.point_3.y
                {
                    (0.0, -self.curve_size * scale_factor).into()
                } else {
                    normalize(
                        stroke_line.point_3 - stroke_line.point_2,
                        self.curve_size * scale_factor,
                    )
                };
                let joint_point_2 = stroke_line.point_2 + delta_vector_2;

                self.cd_draw_line(
                    polygons,
                    stroke_line.point_1,
                    joint_point_1,
                    stroke_line.head_shape,
                    EndType::new(1.0),
                    stroke_line.color,
                );
                self.cd_draw_quadratic_bezier(
                    polygons,
                    joint_point_1,
                    stroke_line.point_2,
                    joint_point_2,
                    EndType::new(1.0),
                    EndType::new(1.0),
                    stroke_line.color,
                );

                if matches!(&stroke_line.tail_shape.kind, EndKind::RightUpwardFlick)
                    && stroke_line.tail_shape.opt == 0
                {
                    let joint_point_3 = (
                        stroke_line.point_3.x - self.curve_size,
                        stroke_line.point_3.y,
                    );
                    let flick_end_point = (
                        stroke_line.point_3.x + self.curve_size * 0.5,
                        stroke_line.point_3.y - self.curve_size * 2.0,
                    );

                    self.cd_draw_line(
                        polygons,
                        joint_point_2,
                        joint_point_3,
                        EndType::new(1.0),
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        joint_point_3,
                        stroke_line.point_3,
                        flick_end_point,
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else {
                    self.cd_draw_line(
                        polygons,
                        joint_point_2,
                        stroke_line.point_3,
                        EndType::new(1.0),
                        stroke_line.tail_shape,
                        stroke_line.color,
                    );
                }
            }
            StrokeKind::ComplexCurve => {
                if matches!(&stroke_line.tail_shape.kind, EndKind::LeftUpwardFlick) {
                    let delta_vector = if stroke_line.point_3.x == stroke_line.point_4.x {
                        (0.0, -self.curve_size).into()
                    } else if stroke_line.point_3.y == stroke_line.point_4.y {
                        (-self.curve_size, 0.0).into()
                    } else {
                        normalize(stroke_line.point_3 - stroke_line.point_4, self.curve_size)
                    };
                    let joint_point = stroke_line.point_4 + delta_vector;

                    self.cd_draw_cubic_bezier(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        stroke_line.point_3,
                        joint_point,
                        stroke_line.head_shape,
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        joint_point,
                        stroke_line.point_4,
                        (
                            stroke_line.point_4.x - self.curve_size * 2.0,
                            stroke_line.point_4.y - self.curve_size * 0.5,
                            false,
                        ),
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else if matches!(&stroke_line.tail_shape.kind, EndKind::RightUpwardFlick)
                    && stroke_line.tail_shape.opt == 0
                {
                    let joint_point = (
                        stroke_line.point_4.x - self.curve_size,
                        stroke_line.point_4.y,
                    );
                    let flick_end_point = (
                        stroke_line.point_4.x + self.curve_size * 0.5,
                        stroke_line.point_4.y - self.curve_size * 2.0,
                    );

                    self.cd_draw_cubic_bezier(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        stroke_line.point_3,
                        joint_point,
                        stroke_line.head_shape,
                        EndType::new(1.0),
                        stroke_line.color,
                    );
                    self.cd_draw_quadratic_bezier(
                        polygons,
                        joint_point,
                        stroke_line.point_4,
                        flick_end_point,
                        EndType::new(1.0),
                        EndType::new(0.0),
                        stroke_line.color,
                    );
                } else {
                    self.cd_draw_cubic_bezier(
                        polygons,
                        stroke_line.point_1,
                        stroke_line.point_2,
                        stroke_line.point_3,
                        stroke_line.point_4,
                        stroke_line.head_shape,
                        stroke_line.tail_shape,
                        stroke_line.color,
                    );
                }
            }
            StrokeKind::VerticalSlash => {
                self.cd_draw_line(
                    polygons,
                    stroke_line.point_1,
                    stroke_line.point_2,
                    stroke_line.head_shape,
                    EndType::new(1.0),
                    stroke_line.color,
                );
                self.cd_draw_quadratic_bezier(
                    polygons,
                    stroke_line.point_2,
                    stroke_line.point_3,
                    stroke_line.point_4,
                    EndType::new(1.0),
                    stroke_line.tail_shape,
                    stroke_line.color,
                );
            }
            // This arm should be reinterpretated as `Line::Unknown` in previous steps.
            _ => {}
        }
    }
}
