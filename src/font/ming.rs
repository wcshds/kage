use core::f64;

use crate::{
    curve::{
        FattenResult, SplitResult, fit_quadratic_bezier, generate_fatten_curve,
        split_quadratic_bezier_curve,
    },
    pen::Pen,
    polygon::Polygon,
    polygons::Polygons,
    stroke::{EndKind, EndType},
    utils::{Point, Vector, is_quadratic, normalize},
};

const DIVIDE_INITIAL_RATE: f64 = 0.5;

struct Ming {
    /// must divide 1000
    k_rate: usize,
    /// Half of the width of mincho-style horizontal (thinner) strokes.
    /// origin name: kMinWidthY
    k_min_width_horizontal: f64,
    /// Determines the size of ウロコ at the 開放 end of mincho-style horizontal strokes.
    /// origin name: kMinWidthU
    k_min_width_triangle: f64,
    /// Half of the width of mincho-style vertical (thicker) strokes.
    /// origin name: kMinWidthT
    k_min_width_vertical: f64,
    /// Half of the width of gothic-style strokes.
    /// Also used to determine the size of mincho's ornamental elements.
    k_width: f64,
    /// Size of kakato in gothic.
    k_square_terminal: f64,
    /// Width at the end of 右払い relative to `2 * kMinWidthT`.
    k_l2rdfatten: f64,
    /// Size of the curve at the end of 左ハネ, and at the middle of 折れ and 乙線 strokes.
    k_mage: f64,
    /// Whether to use off-curve points to approximate curving strokes
    /// with quadratic Bézier curves (experimental).
    k_use_curve: bool,
    /// Length of 左下カド's カカト in mincho for each shortening level (0 to 3) and 413 (左下zh用新).
    /// for KAKATO adjustment 000,100,200,300,400
    k_adjust_kakato_l: Vec<f64>,
    /// Length of 右下カド's カカト in mincho for each shortening level (0 to 3).
    /// for KAKATO adjustment 000,100,200,300
    k_adjust_kakato_r: Vec<f64>,
    /// Width of the collision box below カカト for shortening adjustment.
    /// check area width
    k_adjust_kakato_range_x: f64,
    /// Height of the collision box below カカト for each shortening adjustment level (0 to 3).
    /// 3 steps of checking
    k_adjust_kakato_range_y: Vec<f64>,
    /// f64 of カカト shortening levels. Must be set to 3.
    /// f64 of steps
    k_adjust_kakato_step: f64,
    /// Size of ウロコ at the 開放 end of mincho-style horizontal strokes for each shrinking level (0 to max({@link kAdjustUrokoLengthStep}, {@link kAdjustUroko2Step})).
    /// for UROKO adjustment 000,100,200,300
    k_adjust_uroko_x: Vec<f64>,
    /// Size of ウロコ at the 開放 end of mincho-style horizontal strokes for each shrinking level (0 to max({@link kAdjustUrokoLengthStep}, {@link kAdjustUroko2Step})).
    /// for UROKO adjustment 000,100,200,300
    k_adjust_uroko_y: Vec<f64>,
    /// Threshold length of horizontal strokes for shrinking its ウロコ for each adjustment level ({@link kAdjustUrokoLengthStep} to 1).
    /// length for checking
    k_adjust_uroko_length: Vec<f64>,
    /// f64 of ウロコ shrinking levels by adjustment using collision detection.
    /// f64 of steps
    k_adjust_uroko_length_step: f64,
    /// Size of the collision box to the left of ウロコ at the 開放 end of mincho-style horizontal strokes for each shrinking adjustment level ({@link kAdjustUrokoLengthStep} to 1).
    /// check for crossing. corresponds to length
    k_adjust_uroko_line: Vec<f64>,
    /// f64 of ウロコ shrinking levels by adjustment based on the density of horizontal strokes.
    k_adjust_uroko2_step: f64,
    /// Parameter for shrinking adjustment of ウロコ based on the density of horizontal strokes.
    k_adjust_uroko2_length: f64,
    /// Parameter for thinning adjustment of mincho-style vertical strokes.
    k_adjust_tate_step: f64,
    /// Parameter for thinning adjustment of the latter half of mincho-style 折れ strokes.
    k_adjust_mage_step: f64,
}

impl Ming {
    fn draw_curve_body(
        &self,
        polygons: &mut Polygons,
        start_point: Point,
        control_point_1: Point,
        control_point_2: Point,
        end_point: Point,
        head_shape: EndType,
        tail_shape: EndType,
        min_width_vertical: f64,
        start_width_reduction: f64,
        width_change_rate: f64,
    ) {
        let is_quadratic = is_quadratic(control_point_1, control_point_2);

        if is_quadratic && self.k_use_curve {
            let thinness_ratio = 0.5;
            let width_delta_func = |progress: f64| -> f64 {
                match (&head_shape.kind, &tail_shape.kind) {
                    (&EndKind::Narrow, &EndKind::Free) => progress * thinness_ratio * 1.1,
                    (&EndKind::Narrow, _) => progress * thinness_ratio,
                    (_, &EndKind::Narrow) => (1.0 - progress) * thinness_ratio,
                    _ if start_width_reduction > 0.0 => {
                        // ???
                        let start_reduction = (start_width_reduction / 2.0)
                            / (min_width_vertical - width_change_rate / 2.0);
                        let width_slope = (start_width_reduction / 2.0)
                            / (min_width_vertical - width_change_rate);

                        1.0 - start_reduction + width_slope * progress
                    }
                    _ => 1.0,
                }
            };

            let FattenResult {
                left: left_sampled_points,
                right: right_sampled_points,
            } = generate_fatten_curve(
                start_point,
                control_point_1,
                control_point_2,
                end_point,
                10,
                |progress| {
                    let mut width_delta = (&width_delta_func)(progress);
                    if width_delta < 0.15 {
                        width_delta = 0.15;
                    }
                    min_width_vertical * width_delta
                },
            );

            let SplitResult {
                index: left_sampled_points_actual_divided_index,
                segments:
                    [
                        [_, left_rough_estimate_control_point_1, _],
                        [_, left_rough_estimate_control_point_2, _],
                    ],
            } = split_quadratic_bezier_curve(
                start_point,
                control_point_1,
                end_point,
                &left_sampled_points,
            );
            let SplitResult {
                index: right_sampled_points_actual_divided_index,
                segments:
                    [
                        [_, right_rough_estimate_control_point_1, _],
                        [_, right_rough_estimate_control_point_2, _],
                    ],
            } = split_quadratic_bezier_curve(
                start_point,
                control_point_1,
                end_point,
                &right_sampled_points,
            );

            let left_fitted_1 = fit_quadratic_bezier(
                &left_sampled_points[..left_sampled_points_actual_divided_index + 1],
            );
            let left_fitted_2 = fit_quadratic_bezier(
                &left_sampled_points[left_sampled_points_actual_divided_index..],
            );

            let [left_fitted_1, left_fitted_2] = match (left_fitted_1, left_fitted_2) {
                (Some(left_fitted_1), Some(left_fitted_2)) => [left_fitted_1, left_fitted_2],
                _ => return,
            };

            let mut polygon_1 = Polygon::new(vec![
                left_fitted_1.start_point,
                left_fitted_1.control_point,
                left_fitted_1.end_point,
                left_fitted_2.control_point,
                left_fitted_2.end_point,
            ]);
            let mut polygon_2 = {
                let point_1 = right_sampled_points[0];
                let point_2 = right_rough_estimate_control_point_1
                    - (left_fitted_1.control_point - left_rough_estimate_control_point_1);
                let point_3 = right_sampled_points[right_sampled_points_actual_divided_index];
                let point_4 = right_rough_estimate_control_point_2
                    - (left_fitted_2.control_point - left_rough_estimate_control_point_2);
                let point_5 = right_sampled_points[right_sampled_points.len() - 1];

                Polygon::new::<Point>(vec![
                    (point_1.x, point_1.y, false).into(),
                    (point_2.x, point_2.y, true).into(),
                    (point_3.x, point_3.y, false).into(),
                    (point_4.x, point_4.y, true).into(),
                    (point_5.x, point_5.y, false).into(),
                ])
            };

            polygon_2.reverse();
            polygon_1.concat(polygon_2);
            polygons.push(polygon_1);
        } else {
            let mut thinness_ratio = 0.5;
            let hypot = Vector::from(end_point - start_point).hypot();
            if hypot < 50.0 {
                thinness_ratio += 0.4 * (1.0 - hypot / 50.0);
            }

            let width_delta_func = |progress: f64| -> f64 {
                match (&head_shape.kind, &tail_shape.kind) {
                    (&EndKind::Narrow | &EndKind::RoofedNarrowEntry, &EndKind::Free) => {
                        progress.powf(thinness_ratio) * self.k_l2rdfatten
                    }
                    (&EndKind::Narrow | &EndKind::RoofedNarrowEntry, _) => {
                        if is_quadratic {
                            progress.powf(thinness_ratio)
                        } else {
                            progress.powf(thinness_ratio) * 0.7
                        }
                    }
                    (_, &EndKind::Narrow) => (1.0 - progress) * thinness_ratio,
                    _ if is_quadratic
                        && (start_width_reduction > 0.0 || width_change_rate > 0.0) =>
                    {
                        // ???
                        ((self.k_min_width_vertical - start_width_reduction / 2.0)
                            - (width_change_rate - start_width_reduction) / 2.0 * progress)
                            / self.k_min_width_vertical
                    }
                    _ => 1.0,
                }
            };

            let FattenResult {
                left: left_sampled_points,
                right: right_sampled_points,
            } = generate_fatten_curve(
                start_point,
                control_point_1,
                control_point_2,
                end_point,
                self.k_rate,
                |progress| {
                    let mut width_delta = (&width_delta_func)(progress);

                    if width_delta < 0.15 {
                        width_delta = 0.15;
                    }

                    min_width_vertical * width_delta
                },
            );

            let mut polygon_1 = Polygon::new(left_sampled_points);
            let mut polygon_2 = Polygon::new(right_sampled_points);

            if (matches!(head_shape.kind, EndKind::VerticalConnection) && head_shape.opt_1 == 1)
                || (matches!(tail_shape.kind, EndKind::TopRightCorner) && tail_shape.opt_1 == 0)
                    && ((is_quadratic && start_point.y > end_point.y)
                        || (!is_quadratic && start_point.x > control_point_1.x))
            {
                polygon_1.floor();
                polygon_2.floor();

                for index in 0..polygon_2.len() - 1 {
                    let point_1 = polygon_2.get(index).unwrap();
                    let point_2 = polygon_2.get(index + 1).unwrap();

                    if point_1.y <= start_point.y && start_point.y <= point_2.y {
                        let new_x_1 = point_2.x
                            + (point_1.x - point_2.x) * (start_point.y - point_2.y)
                                / (point_1.y - point_2.y);
                        let new_y_1 = start_point.y;
                        let point_3 = polygon_1.get(0).unwrap();
                        let point_4 = polygon_1.get(1).unwrap();
                        let new_x_2 = if (matches!(head_shape.kind, EndKind::VerticalConnection)
                            && head_shape.opt_1 == 1)
                        {
                            point_3.x
                                + (point_4.x - point_3.x) * (start_point.y - point_3.y)
                                    / (point_4.y - point_3.y)
                        } else {
                            point_3.x
                                + (point_4.x - point_3.x + 1.0) * (start_point.y - point_3.y)
                                    / (point_4.y - point_3.y)
                        };
                        let new_y_2 = if (matches!(head_shape.kind, EndKind::VerticalConnection)
                            && head_shape.opt_1 == 1)
                        {
                            start_point.y
                        } else {
                            start_point.y + 1.0
                        };

                        for _ in 0..index {
                            polygon_2.shift();
                        }

                        polygon_2.set(0, new_x_1, new_y_1, Some(false)).unwrap();
                        polygon_1.unshift(new_x_2, new_y_2, Some(false));
                        break;
                    }
                }
            }

            polygon_2.reverse();
            polygon_1.concat(polygon_2);
            polygons.push(polygon_1);
        }
    }

    fn draw_curve_head(
        &self,
        polygons: &mut Polygons,
        start_point: Point,
        control_point_1: Point,
        head_shape: EndType,
        min_width_vertical: f64,
        is_up_to_bottom: bool,
        corner_offset: f64,
    ) {
        match &head_shape.kind {
            &EndKind::TopLeftCorner => {
                let mut pen = Pen::new(start_point.x, start_point.y);

                if start_point.x != control_point_1.x {
                    // ???
                    pen.set_down(control_point_1.x, control_point_1.y);
                }

                let polygon = pen.get_polygon(&[
                    (-min_width_vertical, 0.0),
                    (min_width_vertical, 0.0),
                    (-min_width_vertical, -min_width_vertical),
                ]);

                polygons.push(polygon);
            }
            &EndKind::Free => {
                if is_up_to_bottom {
                    let mut pen = Pen::new(start_point.x, start_point.y);

                    if start_point.x != control_point_1.x {
                        // ???
                        pen.set_down(control_point_1.x, control_point_1.y);
                    }

                    let mut shape_factor = f64::atan2(
                        (start_point.y - control_point_1.y).abs(),
                        (start_point.x - control_point_1.x).abs(),
                    ) / (f64::consts::PI / 2.0)
                        - 0.4;
                    if shape_factor > 0.0 {
                        shape_factor *= 2.0;
                    } else {
                        shape_factor *= 16.0;
                    }

                    let plus_minus = shape_factor.signum();

                    let polygon_1 = pen.get_polygon(&[
                        (-min_width_vertical, 1.0),
                        (min_width_vertical, 0.0),
                        (
                            -plus_minus * min_width_vertical,
                            -self.k_min_width_horizontal * shape_factor.abs(),
                        ),
                    ]);
                    polygons.push(polygon_1);

                    let move_offset = if shape_factor < 0.0 {
                        -shape_factor * self.k_min_width_horizontal
                    } else {
                        0.0
                    };
                    let polygon_2 = pen.get_polygon(&if start_point.x == control_point_1.x
                        && start_point.y == control_point_1.y
                    {
                        [
                            (min_width_vertical, -move_offset),
                            (
                                min_width_vertical * 1.5,
                                self.k_min_width_horizontal - move_offset,
                            ),
                            (
                                min_width_vertical - 2.0,
                                self.k_min_width_horizontal * 2.0 + 1.0,
                            ),
                        ]
                    } else {
                        [
                            (min_width_vertical, -move_offset),
                            (
                                min_width_vertical * 1.5,
                                self.k_min_width_horizontal - move_offset * 1.2,
                            ),
                            (
                                min_width_vertical - 2.0,
                                self.k_min_width_horizontal * 2.0 - move_offset * 0.8 + 1.0,
                            ),
                        ]
                    });
                    polygons.push(polygon_2);
                } else {
                    let mut pen = Pen::new(start_point.x, start_point.y);

                    if start_point.x == control_point_1.x {
                        pen.set_matrix2(0.0, 1.0);
                    } else {
                        pen.set_right(control_point_1.x, control_point_1.y);
                    }

                    let polygon_1 = pen.get_polygon(&[
                        (0.0, min_width_vertical),
                        (0.0, -min_width_vertical),
                        (-self.k_min_width_horizontal, -min_width_vertical),
                    ]);
                    polygons.push(polygon_1);

                    let polygon_2 = pen.get_polygon(&[
                        (0.0, min_width_vertical),
                        (self.k_min_width_horizontal, min_width_vertical * 1.5),
                        (self.k_min_width_horizontal * 3.0, min_width_vertical * 0.5),
                    ]);
                    polygons.push(polygon_2);
                }
            }
            &EndKind::TopRightCorner | &EndKind::RoofedNarrowEntry => {
                let pen = Pen::new(start_point.x - corner_offset, start_point.y);
                let mut polygon_1 = pen.get_polygon(&[
                    (-min_width_vertical, -self.k_min_width_horizontal),
                    (0.0, -self.k_min_width_horizontal - self.k_width),
                    (
                        min_width_vertical + self.k_width,
                        self.k_min_width_horizontal,
                    ),
                    (min_width_vertical, self.k_min_width_vertical - 1.0),
                ]);
                let polygon_2 = if matches!(&head_shape.kind, &EndKind::RoofedNarrowEntry) {
                    Polygon::new::<Point>(vec![
                        (0.0, self.k_min_width_vertical + 2.0).into(),
                        (0.0, 0.0).into(),
                    ])
                } else {
                    Polygon::new::<Point>(vec![
                        (-self.k_min_width_vertical, self.k_min_width_vertical + 4.0).into(),
                    ])
                };

                polygon_1.concat(polygon_2);
                polygons.push(polygon_1);
            }
            _ => {}
        }
    }

    fn draw_curve_tail(
        &self,
        polygons: &mut Polygons,
        control_point_2: Point,
        end_point: Point,
        head_shape: EndType,
        tail_shape: EndType,
        min_width_vertical: f64,
        hane_adjustment: f64,
        tail_circle_adjustment: f64,
        is_bottom_to_up: bool,
    ) {
        match [&head_shape.kind, &tail_shape.kind] {
            [_, &EndKind::Temp1 | &EndKind::Stop | &EndKind::Temp15] => {
                let min_width_vertical_new =
                    self.k_min_width_vertical - tail_circle_adjustment / 2.0;

                let mut pen = Pen::new(end_point.x, end_point.y);
                if control_point_2.x == end_point.x {
                    pen.set_matrix2(0.0, 1.0);
                } else if control_point_2.y != end_point.y {
                    pen.set_left(control_point_2.x, control_point_2.y);
                }

                let local_points = if self.k_use_curve {
                    [
                        (0.0, -min_width_vertical_new, false),
                        (
                            min_width_vertical_new * 0.9,
                            -min_width_vertical_new * 0.9,
                            true,
                        )
                            .into(),
                        (min_width_vertical_new, 0.0, false),
                        (
                            min_width_vertical_new * 0.9,
                            min_width_vertical_new * 0.9,
                            true,
                        )
                            .into(),
                        (0.0, min_width_vertical_new, false),
                    ]
                } else {
                    [
                        (0.0, -min_width_vertical_new, false),
                        (
                            min_width_vertical_new * 0.7,
                            -min_width_vertical_new * 0.7,
                            false, // ???
                        ),
                        (min_width_vertical_new, 0.0, false),
                        (
                            min_width_vertical_new * 0.7,
                            min_width_vertical_new * 0.7,
                            false, // ???
                        ),
                        (0.0, min_width_vertical_new, false),
                    ]
                };
                let mut polygon = pen.get_polygon(&local_points);

                if control_point_2.x == end_point.x {
                    polygon.reverse();
                }
                polygons.push(polygon);

                if matches!(&tail_shape.kind, &EndKind::Temp15) {
                    let mut pen = Pen::new(end_point.x, end_point.y);

                    if is_bottom_to_up {
                        pen.set_matrix2(-1.0, 0.0);
                    }

                    let polygon = pen.get_polygon(&[
                        (0.0, -min_width_vertical + 1.0).into(),
                        (2.0, -min_width_vertical - self.k_width * 5.0),
                        (0.0, -min_width_vertical - self.k_width * 5.0),
                        (-min_width_vertical, -min_width_vertical + 1.0),
                    ]);

                    polygons.push(polygon);
                }
            }
            [
                &EndKind::Narrow | &EndKind::RoofedNarrowEntry,
                &EndKind::Free,
            ]
            | [_, &EndKind::Temp9] => {
                let mut shape_factor = f64::atan2(
                    (end_point.y - control_point_2.y).abs(),
                    (end_point.x - control_point_2.x).abs(),
                ) / (f64::consts::PI / 2.0)
                    - 0.6;
                if shape_factor > 0.0 {
                    shape_factor *= 8.0;
                } else {
                    shape_factor *= 3.0;
                }

                let plus_minus = shape_factor.signum();

                let mut pen = Pen::new(end_point.x, end_point.y);
                if control_point_2.y == end_point.y {
                    pen.set_matrix2(1.0, 0.0);
                } else if control_point_2.x == end_point.x {
                    let sin_theta = if end_point.y > control_point_2.y {
                        -1.0
                    } else {
                        1.0
                    };
                    pen.set_matrix2(0.0, sin_theta);
                } else {
                    pen.set_left(control_point_2.x, control_point_2.y);
                }

                let polygon = pen.get_polygon(&[
                    (0.0, min_width_vertical * self.k_l2rdfatten),
                    (0.0, -min_width_vertical * self.k_l2rdfatten),
                    (
                        shape_factor.abs() * min_width_vertical * self.k_l2rdfatten,
                        plus_minus * min_width_vertical * self.k_l2rdfatten,
                    ),
                ]);
                polygons.push(polygon);
            }
            [_, &EndKind::Temp14] => {
                // const jumpFactor = kMinWidthT > 6 ? 6.0 / kMinWidthT : 1.0;
                // const haneLength = font.kWidth * 4 * Math.min(1 - haneAdjustment / 10, (kMinWidthT / font.kMinWidthT) ** 3) * jumpFactor;
                // const poly = new Pen(x2, y2).getPolygon([
                //     { x: 0, y: 0 },
                //     { x: 0, y: -kMinWidthT },
                //     { x: -haneLength, y: -kMinWidthT },
                //     { x: -haneLength, y: -kMinWidthT * 0.5 },
                // ]);
                // // poly.reverse();
                // polygons.push(poly);
                let jump_factor = if min_width_vertical > 6.0 {
                    6.0 / min_width_vertical
                } else {
                    1.0
                };
                let hane_length = self.k_width
                    * 4.0
                    * f64::min(
                        1.0 - hane_adjustment / 10.0,
                        (min_width_vertical / self.k_min_width_vertical).powi(3),
                    )
                    * jump_factor;
                let pen = Pen::new(end_point.x, end_point.y);
                let polygon = pen.get_polygon(&[
                    (0.0, 0.0),
                    (0.0, -min_width_vertical),
                    (-hane_length, -min_width_vertical),
                    (-hane_length, -min_width_vertical * 0.5),
                ]);
                polygons.push(polygon);
            }
            _ => {}
        }
    }

    fn cd_draw_curve_universal(
        &self,
        polygons: &mut Polygons,
        start_point: Point,
        control_point_1: Point,
        control_point_2: Point,
        end_point: Point,
        head_shape: EndType,
        tail_shape: EndType,
        vertical_thickness_adjustment: f64,
        hane_adjustment: f64,
        start_thickness_adjustment: f64,
        end_thickness_adjustment: f64,
    ) {
        let min_width_vertical = self.k_min_width_vertical - vertical_thickness_adjustment / 2.0;

        let mut start_point = start_point;
        let mut delta_1 = None;
        match &head_shape.kind {
            &EndKind::Free | &EndKind::Narrow | &EndKind::RoofedNarrowEntry => {
                delta_1 = Some(-1.0 * self.k_min_width_horizontal * 0.5);
            }
            &EndKind::Temp1
            | &EndKind::HorizontalConnection
            | &EndKind::Temp6
            | &EndKind::TopRightCorner
            | &EndKind::VerticalConnection => {
                delta_1 = Some(0.0);
            }
            &EndKind::TopLeftCorner => {
                delta_1 = Some(self.k_min_width_horizontal);
            }
            _ => {}
        }

        if let Some(delta) = delta_1
            && delta != 0.0
        {
            let delta_vector =
                if start_point.x == control_point_1.x && start_point.y == control_point_1.y {
                    (0.0, delta).into()
                } else {
                    normalize(start_point - control_point_1, delta)
                };
            start_point = start_point + delta_vector;
        }

        let mut corner_offset = 0.0;
        if let Some(_) = delta_1
            && matches!(
                &head_shape.kind,
                &EndKind::TopRightCorner | &EndKind::RoofedNarrowEntry
            )
            && matches!(&tail_shape.kind, &EndKind::Narrow)
            && min_width_vertical > 6.0
        {
            let contour_length = Vector::from(control_point_1 - start_point).hypot()
                + Vector::from(control_point_2 - control_point_1).hypot()
                + Vector::from(end_point - control_point_2).hypot();

            if contour_length < 100.0 {
                corner_offset = (min_width_vertical - 6.0) * ((100.0 - contour_length) / 100.0);
                start_point.x += corner_offset;
            }
        }

        let mut end_point = end_point;
        let mut delta_2 = None;
        match &tail_shape.kind {
            &EndKind::Free
            | &EndKind::Temp1
            | &EndKind::Narrow
            | &EndKind::Temp9
            | &EndKind::Temp14
            | &EndKind::Temp15
            | &EndKind::Temp17
            | &EndKind::RightUpwardFlick => {
                delta_2 = Some(0.0);
            }
            &EndKind::Stop => {
                delta_2 = Some(-1.0 * min_width_vertical * 0.5);
            }
            _ => {
                delta_2 = delta_1;
            }
        }

        if let Some(delta) = delta_2
            && delta != 0.0
        {
            let delta_vector =
                if control_point_2.x == end_point.x && control_point_2.y == end_point.y {
                    (0.0, -delta).into()
                } else {
                    normalize(end_point - control_point_2, delta)
                };
            end_point = end_point + delta_vector;
        }

        if let Some(_) = delta_1
            && let Some(_) = delta_2
        {
            self.draw_curve_body(
                polygons,
                start_point,
                control_point_1,
                control_point_2,
                end_point,
                head_shape,
                tail_shape,
                min_width_vertical,
                start_thickness_adjustment,
                end_thickness_adjustment,
            );
        }

        if let Some(_) = delta_1 {
            let is_up_to_bottom = if matches!(delta_2, None) {
                false
            } else {
                start_point.y <= end_point.y
            };

            self.draw_curve_head(
                polygons,
                start_point,
                control_point_1,
                head_shape,
                min_width_vertical,
                is_up_to_bottom,
                corner_offset,
            );
        }

        if let Some(_) = delta_2 {
            let is_bottom_to_up = if matches!(delta_1, None) {
                false
            } else {
                end_point.y <= start_point.y
            };

            self.draw_curve_tail(
                polygons,
                control_point_2,
                end_point,
                head_shape,
                tail_shape,
                min_width_vertical,
                hane_adjustment,
                end_thickness_adjustment,
                is_bottom_to_up,
            );
        }
    }

    fn cd_draw_quadratic_bezier(
        &self,
        polygons: &mut Polygons,
        start_point: Point,
        control_point: Point,
        end_point: Point,
        head_shape: EndType,
        tail_shape: EndType,
        vertical_thickness_adjustment: f64,
        hane_adjustment: f64,
        start_thickness_adjustment: f64,
        end_thickness_adjustment: f64,
    ) {
        self.cd_draw_curve_universal(
            polygons,
            start_point,
            control_point,
            control_point,
            end_point,
            head_shape,
            tail_shape,
            vertical_thickness_adjustment,
            hane_adjustment,
            start_thickness_adjustment,
            end_thickness_adjustment,
        )
    }

    fn cd_draw_cubic_bezier(
        &self,
        polygons: &mut Polygons,
        start_point: Point,
        control_point_1: Point,
        control_point_2: Point,
        end_point: Point,
        head_shape: EndType,
        tail_shape: EndType,
        vertical_thickness_adjustment: f64,
        hane_adjustment: f64,
        start_thickness_adjustment: f64,
        end_thickness_adjustment: f64,
    ) {
        self.cd_draw_curve_universal(
            polygons,
            start_point,
            control_point_1,
            control_point_2,
            end_point,
            head_shape,
            tail_shape,
            vertical_thickness_adjustment,
            hane_adjustment,
            start_thickness_adjustment,
            end_thickness_adjustment,
        )
    }

    fn cd_draw_line(
        &self,
        polygons: &mut Polygons,
        start_point: Point,
        end_point: Point,
        head_shape: EndType,
        tail_shape: EndType,
        vertical_thickness_adjustment: f64,
        triangle_adjustment: f64,
        square_adjustment: usize,
    ) {
        let min_width_vertical = self.k_min_width_vertical - vertical_thickness_adjustment / 2.0;

        if start_point.x == end_point.x
            || start_point.y != end_point.y
                && (start_point.x > end_point.x
                    || (end_point.y - start_point.y).abs() >= (end_point.x - start_point.x).abs() // The angle is very steep, with a large vertical component.
                    || matches!(&head_shape.kind, &EndKind::Temp6)
                    || matches!(&tail_shape.kind, &EndKind::Temp6))
        {
            // if vertical stroke, use y-axis; for others, use x-axis.
            let (cos_radian, sin_radian) = if start_point.x == end_point.x {
                (0.0, 1.0)
            } else {
                let vector = normalize(end_point - start_point, 1.0);
                (vector.x, vector.y)
            };

            let mut pen_1 = Pen::new(start_point.x, start_point.y);
            let mut pen_2 = Pen::new(end_point.x, end_point.y);
            pen_1.set_matrix2(sin_radian, -cos_radian);
            pen_2.set_matrix2(sin_radian, -cos_radian);

            let mut polygon = Polygon::new_with_length(4);
            match &head_shape.kind {
                &EndKind::Free => {
                    polygon
                        .set_point(
                            0,
                            pen_1.get_point(
                                min_width_vertical,
                                self.k_min_width_horizontal / 2.0,
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                    polygon
                        .set_point(
                            3,
                            pen_1.get_point(
                                -min_width_vertical,
                                -self.k_min_width_horizontal / 2.0,
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                }
                &EndKind::Temp1 | &EndKind::Temp6 => {
                    polygon
                        .set_point(0, pen_1.get_point(min_width_vertical, 0.0, false))
                        .expect("The length of polygon is equal to 4.");
                    polygon
                        .set_point(3, pen_1.get_point(-min_width_vertical, 0.0, false))
                        .expect("The length of polygon is equal to 4.");
                }
                &EndKind::TopLeftCorner => {
                    polygon
                        .set_point(
                            0,
                            pen_1.get_point(
                                min_width_vertical,
                                -self.k_min_width_horizontal,
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                    polygon
                        .set_point(
                            3,
                            pen_1.get_point(
                                -min_width_vertical,
                                -self.k_min_width_horizontal - min_width_vertical,
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                }
                &EndKind::TopRightCorner => {
                    if start_point.x == end_point.x {
                        polygon
                            .set(
                                0,
                                start_point.x + min_width_vertical,
                                start_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set(
                                3,
                                start_point.x - min_width_vertical,
                                start_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                    } else {
                        let v = if start_point.x > end_point.x {
                            -1.0
                        } else {
                            1.0
                        };
                        polygon
                            .set(
                                0,
                                start_point.x + (min_width_vertical + v) / sin_radian,
                                start_point.y + 1.0,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set(
                                3,
                                start_point.x - min_width_vertical / sin_radian,
                                start_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                    }
                }
                &EndKind::VerticalConnection => {
                    if start_point.x == end_point.x {
                        polygon
                            .set(
                                0,
                                start_point.x + min_width_vertical,
                                start_point.y - self.k_min_width_horizontal,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set(
                                3,
                                start_point.x - min_width_vertical,
                                start_point.y - self.k_min_width_horizontal,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                    } else {
                        polygon
                            .set(
                                0,
                                start_point.x + min_width_vertical / sin_radian,
                                start_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set(
                                3,
                                start_point.x - min_width_vertical / sin_radian,
                                start_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                    }
                }
                _ => {}
            }

            match &tail_shape.kind {
                &EndKind::Free => {
                    if matches!(&head_shape.kind, &EndKind::Temp6) {
                        polygon
                            .set_point(1, pen_2.get_point(min_width_vertical, 0.0, false))
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set_point(2, pen_2.get_point(-min_width_vertical, 0.0, false))
                            .expect("The length of polygon is equal to 4.");
                    } else {
                        polygon
                            .set_point(
                                1,
                                pen_2.get_point(
                                    min_width_vertical,
                                    -min_width_vertical / 2.0,
                                    false,
                                ),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set_point(2, pen_2.get_point(-min_width_vertical, 0.0, false))
                            .expect("The length of polygon is equal to 4.");
                    }
                }
                &EndKind::RightUpwardFlick if start_point.x == end_point.x => {}
                &EndKind::RightUpwardFlick | &EndKind::Temp1 => {
                    polygon
                        .set_point(1, pen_2.get_point(min_width_vertical, 0.0, false))
                        .expect("The length of polygon is equal to 4.");
                    polygon
                        .set_point(2, pen_2.get_point(-min_width_vertical, 0.0, false))
                        .expect("The length of polygon is equal to 4.");
                }
                &EndKind::BottomLeftCorner => {
                    polygon
                        .set_point(
                            1,
                            pen_2.get_point(
                                min_width_vertical,
                                self.k_adjust_kakato_l[square_adjustment],
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                    polygon
                        .set_point(
                            2,
                            pen_2.get_point(
                                -min_width_vertical,
                                self.k_adjust_kakato_l[square_adjustment] + min_width_vertical,
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                }
                &EndKind::BottomRightCorner => {
                    polygon
                        .set_point(
                            1,
                            pen_2.get_point(
                                min_width_vertical,
                                self.k_adjust_kakato_r[square_adjustment],
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                    polygon
                        .set_point(
                            2,
                            pen_2.get_point(
                                -min_width_vertical,
                                self.k_adjust_kakato_r[square_adjustment] + min_width_vertical,
                                false,
                            ),
                        )
                        .expect("The length of polygon is equal to 4.");
                }
                &EndKind::BottomRightHorT | &EndKind::VerticalConnection => {
                    if start_point.x == end_point.x {
                        polygon
                            .set(
                                1,
                                end_point.x + min_width_vertical,
                                end_point.y + self.k_min_width_horizontal,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set(
                                2,
                                end_point.x - min_width_vertical,
                                end_point.y + self.k_min_width_horizontal,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                    } else {
                        polygon
                            .set(
                                1,
                                end_point.x + min_width_vertical / sin_radian, // ???
                                end_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                        polygon
                            .set(
                                2,
                                end_point.x - min_width_vertical / sin_radian, // ???
                                end_point.y,
                                Some(false),
                            )
                            .expect("The length of polygon is equal to 4.");
                    }
                }
                _ => {}
            }

            polygons.push(polygon);

            match &tail_shape.kind {
                &EndKind::BottomRightHorT => {
                    let pen = Pen::new(end_point.x, end_point.y);
                    let polygon = pen.get_polygon(&[
                        (0.0, self.k_min_width_horizontal, false),
                        if start_point.x == end_point.x {
                            (
                                min_width_vertical,
                                -self.k_min_width_horizontal * 3.0,
                                false,
                            )
                        } else {
                            (
                                min_width_vertical * 0.5,
                                -self.k_min_width_horizontal * 4.0,
                                false,
                            )
                        },
                        (
                            self.k_min_width_vertical * 2.0,
                            -self.k_min_width_horizontal,
                            false,
                        ),
                        (
                            self.k_min_width_vertical * 2.0,
                            self.k_min_width_horizontal,
                            false,
                        ),
                    ]);
                    polygons.push(polygon);
                }
                &EndKind::BottomLeftZhNew => {
                    let offset = if start_point.x > end_point.x && start_point.y != end_point.y {
                        ((start_point.x - end_point.x) / (end_point.y - start_point.y) * 3.0)
                            .floor()
                    } else {
                        0.0
                    };

                    let pen = Pen::new(end_point.x + offset, end_point.y);
                    let polygon = pen.get_polygon(&[
                        (0.0, -self.k_min_width_horizontal * 5.0),
                        (-min_width_vertical * 2.0, 0.0),
                        (
                            -self.k_min_width_horizontal,
                            self.k_min_width_horizontal * 5.0,
                        ),
                        (min_width_vertical, self.k_min_width_horizontal),
                        (0.0, 0.0),
                    ]);
                    polygons.push(polygon);
                }
                _ => {}
            }

            match &head_shape.kind {
                &EndKind::TopRightCorner => {
                    // keep the angle of the wedge unchanged even if the stroke is oblique
                    let pen = Pen::new(start_point.x, start_point.y);
                    let mut polygon_1 = pen.get_polygon(&[
                        (-min_width_vertical, -self.k_min_width_horizontal, false),
                        (0.0, -self.k_min_width_horizontal - self.k_width, false),
                        (
                            min_width_vertical + self.k_width,
                            self.k_min_width_horizontal,
                            false,
                        ),
                    ]);
                    let polygon_2 = if start_point.x == end_point.x {
                        Polygon::new(vec![
                            (min_width_vertical, min_width_vertical, false),
                            (-min_width_vertical, 0.0, false),
                        ])
                    } else {
                        Polygon::new(vec![
                            (min_width_vertical, min_width_vertical - 1.0, false),
                            (-min_width_vertical, min_width_vertical + 4.0, false),
                        ])
                    };
                    polygon_1.concat(polygon_2);
                    polygons.push(polygon_1);
                }
                &EndKind::RoofedNarrowEntry => {
                    // keep the angle of the wedge unchanged even if the stroke is oblique
                    let pen = Pen::new(start_point.x, start_point.y);
                    let mut polygon_1 = pen.get_polygon(&[
                        (-min_width_vertical, -self.k_min_width_horizontal, false),
                        (0.0, -self.k_min_width_horizontal - self.k_width, false),
                        (
                            min_width_vertical + self.k_width,
                            self.k_min_width_horizontal,
                            false,
                        ),
                    ]);
                    let polygon_2 = if start_point.x == end_point.x {
                        Polygon::new(vec![
                            (min_width_vertical, min_width_vertical, false),
                            (-min_width_vertical, 0.0, false),
                        ])
                    } else {
                        Polygon::new(vec![
                            (min_width_vertical, min_width_vertical - 1.0, false),
                            (0.0, min_width_vertical + 2.0, false),
                            (0.0, 0.0, false),
                        ])
                    };
                    polygon_1.concat(polygon_2);
                    polygons.push(polygon_1);
                }
                &EndKind::Free => {
                    let polygon = pen_1.get_polygon(&[
                        (min_width_vertical, self.k_min_width_horizontal * 0.5, false),
                        (
                            min_width_vertical * 1.5,
                            self.k_min_width_horizontal * 1.5,
                            false,
                        ),
                        if start_point.x != end_point.x {
                            (
                                start_point.x
                                    + (self.k_min_width_vertical - 2.0) * sin_radian
                                    + (self.k_min_width_horizontal * 2.5) * cos_radian,
                                start_point.y
                                    + (self.k_min_width_vertical + 1.0) * (-cos_radian)
                                    + (self.k_min_width_horizontal * 2.5) * sin_radian,
                                false,
                            )
                        } else {
                            (
                                min_width_vertical - 2.0,
                                self.k_min_width_horizontal * 2.5 + 1.0,
                                false,
                            )
                        },
                    ]);
                    polygons.push(polygon);
                }
                _ => {}
            }

            if (start_point.x == end_point.x && matches!(&tail_shape.kind, &EndKind::Temp1))
                || (matches!(&head_shape.kind, &EndKind::Temp6)
                    && (matches!(&tail_shape.kind, &EndKind::Free)
                        || (start_point.x != end_point.x
                            && matches!(&tail_shape.kind, &EndKind::RightUpwardFlick))))
            {
                let mut polygon = Polygon::new_empty();
                if self.k_use_curve {
                    polygon.push_point(pen_2.get_point(min_width_vertical, 0.0, false));
                    polygon.push(
                        end_point.x - cos_radian * min_width_vertical * 0.9
                            + (-sin_radian) * (-min_width_vertical) * 0.9,
                        end_point.y
                            + sin_radian * min_width_vertical * 0.9
                            + cos_radian * (-min_width_vertical) * 0.9,
                        Some(true),
                    );
                    polygon.push_point(pen_2.get_point(0.0, min_width_vertical, false));
                    polygon.push_point(pen_2.get_point(
                        (-min_width_vertical) * 0.9,
                        min_width_vertical * 0.9,
                        true,
                    ));
                    polygon.push_point(pen_2.get_point(-min_width_vertical, 0.0, false));
                } else {
                    let r = if start_point.x == end_point.x
                        && ((matches!(&head_shape.kind, &EndKind::Temp6)
                            && matches!(&tail_shape.kind, &EndKind::Free))
                            || matches!(&tail_shape.kind, &EndKind::Temp1))
                    {
                        0.6
                    } else {
                        0.8
                    };

                    polygon.push_point(pen_2.get_point(min_width_vertical, 0.0, false));
                    polygon.push_point(pen_2.get_point(
                        min_width_vertical * 0.6,
                        min_width_vertical * r,
                        false,
                    ));
                    polygon.push_point(pen_2.get_point(0.0, min_width_vertical, false));
                    polygon.push_point(pen_2.get_point(
                        (-min_width_vertical) * 0.6,
                        min_width_vertical * r,
                        false,
                    ));
                    polygon.push_point(pen_2.get_point(-min_width_vertical, 0.0, false));

                    if start_point.x == end_point.x
                        && ((matches!(&head_shape.kind, &EndKind::Temp6)
                            && matches!(&tail_shape.kind, &EndKind::Free))
                            || matches!(&tail_shape.kind, &EndKind::Temp1))
                    {
                        polygon.reverse();
                    }
                    polygons.push(polygon);
                    if start_point.x != end_point.x
                        && matches!(&head_shape.kind, &EndKind::Temp6)
                        && matches!(&tail_shape.kind, &EndKind::RightUpwardFlick)
                    {
                        let hane_length = self.k_width * 5.0;
                        let rv = if start_point.x < end_point.x {
                            1.0
                        } else {
                            -1.0
                        };
                        let polygon = pen_2.get_polygon(&[
                            (rv * (min_width_vertical - 1.0), 0.0, false),
                            (rv * (min_width_vertical + hane_length), 2.0, false),
                            (rv * (min_width_vertical + hane_length), 0.0, false),
                            (min_width_vertical - 1.0, -min_width_vertical, false),
                        ]);
                        polygons.push(polygon);
                    }
                }
            }
        } else if start_point.y == end_point.y && matches!(&head_shape.kind, &EndKind::Temp6) {
            let pen_1 = Pen::new(start_point.x, start_point.y);
            let pen_2 = Pen::new(end_point.x, end_point.y);
            let polygon = Polygon::new(vec![
                pen_1.get_point(0.0, -min_width_vertical, false),
                pen_2.get_point(0.0, -min_width_vertical, false),
                pen_2.get_point(0.0, min_width_vertical, false),
                pen_1.get_point(0.0, min_width_vertical, false),
            ]);
            polygons.push(polygon);

            match &tail_shape.kind {
                &EndKind::Temp1 | &EndKind::Free | &EndKind::RightUpwardFlick => {
                    let mut pen = Pen::new(end_point.x, end_point.y);
                    if start_point.x > end_point.x {
                        pen.set_matrix2(-1.0, 0.0);
                    }
                    // const r = 0.6;
                    // const poly = pen2.getPolygon(
                    //     (font.kUseCurve)
                    //         ? [
                    //             { x: 0, y: -kMinWidthT },
                    //             { x: +kMinWidthT * 0.9, y: -kMinWidthT * 0.9, off: true },
                    //             { x: +kMinWidthT, y: 0 },
                    //             { x: +kMinWidthT * 0.9, y: +kMinWidthT * 0.9, off: true },
                    //             { x: 0, y: +kMinWidthT },
                    //         ]
                    //         : [
                    //             { x: 0, y: -kMinWidthT },
                    //             { x: +kMinWidthT * r, y: -kMinWidthT * 0.6 },
                    //             { x: +kMinWidthT, y: 0 },
                    //             { x: +kMinWidthT * r, y: +kMinWidthT * 0.6 },
                    //             { x: 0, y: +kMinWidthT },
                    //         ]);
                    // if (x1 >= x2) {
                    //     poly.reverse();
                    // }
                    // polygons.push(poly);

                    // if (a2 === 5) {
                    //     const haneLength = font.kWidth * (4 * (1 - opt1 / font.kAdjustMageStep) + 1);
                    //     // KAGI NO YOKO BOU NO HANE
                    //     const rv = x1 < x2 ? 1 : -1;
                    //     const poly = pen2.getPolygon([
                    //         // { x: 0, y: rv * (-kMinWidthT + 1) },
                    //         { x: 0, y: rv * -kMinWidthT },
                    //         { x: 2, y: rv * (-kMinWidthT - haneLength) },
                    //         { x: 0, y: rv * (-kMinWidthT - haneLength) },
                    //         // { x: -kMinWidthT, y: rv * (-kMinWidthT + 1) },
                    //         { x: -kMinWidthT, y: rv * -kMinWidthT },
                    //     ]);
                    //     // poly2.reverse(); // for fill-rule
                    //     polygons.push(poly);
                    // }
                    let r = 0.6;
                    let local_points = if self.k_use_curve {
                        vec![
                            (0.0, -min_width_vertical, false),
                            (min_width_vertical * 0.9, -min_width_vertical * 0.9, true),
                            (min_width_vertical, 0.0, false),
                            (min_width_vertical * 0.9, min_width_vertical * 0.9, true),
                            (0.0, min_width_vertical, false),
                        ]
                    } else {
                        vec![
                            (0.0, -min_width_vertical, false),
                            (min_width_vertical * r, -min_width_vertical * 0.6, false),
                            (min_width_vertical, 0.0, false),
                            (min_width_vertical * r, min_width_vertical * 0.6, false),
                            (0.0, min_width_vertical, false),
                        ]
                    };
                    let mut polygon = pen.get_polygon(&local_points);
                    if start_point.x > end_point.x {
                        polygon.reverse();
                    }
                    polygons.push(polygon);

                    if matches!(&tail_shape.kind, &EndKind::RightUpwardFlick) {
                        let hane_length = self.k_width
                            * (4.0
                                * (1.0 - vertical_thickness_adjustment / self.k_adjust_mage_step)
                                + 1.0);
                        let rv = if start_point.x < end_point.x {
                            1.0
                        } else {
                            -1.0
                        };
                        let polygon = pen.get_polygon(&[
                            (0.0, rv * (-min_width_vertical), false),
                            (2.0, rv * (-min_width_vertical - hane_length), false),
                            (0.0, rv * (-min_width_vertical - hane_length), false),
                            (-min_width_vertical, rv * (-min_width_vertical), false),
                        ]);
                        polygons.push(polygon);
                    }
                }
                _ => {}
            }
        } else {
            let (cos_radian, sin_radian) = if start_point.y == end_point.y {
                (1.0, 0.0)
            } else {
                let vector = normalize(end_point - start_point, 1.0);
                (vector.x, vector.y)
            };

            let mut pen_1 = Pen::new(start_point.x, start_point.y);
            let mut pen_2 = Pen::new(end_point.x, end_point.y);
            pen_1.set_matrix2(cos_radian, sin_radian);
            pen_2.set_matrix2(cos_radian, sin_radian);

            let mut polygon = Polygon::new(vec![
                pen_1.get_point(0.0, -self.k_min_width_horizontal, false),
                pen_2.get_point(0.0, -self.k_min_width_horizontal, false),
                pen_2.get_point(0.0, self.k_min_width_horizontal, false),
                pen_1.get_point(0.0, self.k_min_width_horizontal, false),
            ]);
            polygons.push(polygon);

            match &tail_shape.kind {
                // triangle terminal
                &EndKind::Free => {
                    let square_terminal_scale =
                        (self.k_min_width_triangle / self.k_min_width_horizontal - 1.0) / 4.0 + 1.0;
                    let mut polygon_2 = pen_2.get_polygon(&[
                        (0.0, -self.k_min_width_horizontal, false),
                        (
                            -self.k_adjust_uroko_x[square_adjustment] * square_terminal_scale,
                            0.0,
                            false,
                        ),
                    ]);
                    polygon_2.push(
                        end_point.x
                            - (cos_radian - sin_radian) * self.k_adjust_uroko_x[square_adjustment]
                                / 2.0,
                        end_point.y
                            - (sin_radian + cos_radian)
                                * self.k_adjust_uroko_y[square_adjustment]
                                * square_terminal_scale,
                        Some(false),
                    );
                    polygons.push(polygon_2);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{font::ming::Ming, polygons::Polygons, stroke::EndType};

    fn init(use_curve: bool) -> Ming {
        Ming {
            k_rate: 100,
            k_min_width_horizontal: 2.0,
            k_min_width_triangle: 2.0,
            k_min_width_vertical: 6.0,
            k_width: 5.0,
            k_square_terminal: 3.0,
            k_l2rdfatten: 1.1,
            k_mage: 10.0,
            k_use_curve: use_curve,
            k_adjust_kakato_l: vec![14.0, 9.0, 5.0, 2.0, 0.0],
            k_adjust_kakato_r: vec![8.0, 6.0, 4.0, 2.0],
            k_adjust_kakato_range_x: 20.0,
            k_adjust_kakato_range_y: vec![1.0, 19.0, 24.0, 30.0],
            k_adjust_kakato_step: 3.0,
            k_adjust_uroko_x: vec![24.0, 20.0, 16.0, 12.0],
            k_adjust_uroko_y: vec![12.0, 11.0, 9.0, 8.0],
            k_adjust_uroko_length: vec![22.0, 36.0, 50.0],
            k_adjust_uroko_length_step: 3.0,
            k_adjust_uroko_line: vec![22.0, 26.0, 30.0],
            k_adjust_uroko2_step: 3.0,
            k_adjust_uroko2_length: 40.0,
            k_adjust_tate_step: 4.0,
            k_adjust_mage_step: 5.0,
        }
    }

    #[test]
    fn test_draw_curve_body() {
        let ming = init(true);
        let mut polygons = Polygons::new();

        ming.draw_curve_body(
            &mut polygons,
            (151.0, 23.0).into(),
            (146.0, 178.0).into(),
            (146.0, 178.0).into(),
            (182.0, 182.0).into(),
            EndType::new(22.0),
            EndType::new(15.0),
            6.0,
            0.0,
            0.0,
        );

        println!("{}", polygons.generate_svg(true));
    }

    #[test]
    fn test_draw_curve_head() {
        let ming = init(false);
        let mut polygons = Polygons::new();

        ming.draw_curve_head(
            &mut polygons,
            (56.0, 62.0).into(),
            (191.0, 62.0).into(),
            EndType::new(0.0),
            6.0,
            true,
            0.0,
        );

        println!("{:#?}", polygons);
    }

    #[test]
    fn test_draw_curve_tail() {
        let ming = init(false);
        let mut polygons = Polygons::new();

        ming.draw_curve_tail(
            &mut polygons,
            (106.0, 115.0).into(),
            (157.0, 162.0).into(),
            EndType::new(7.0),
            EndType::new(15.0),
            6.0,
            0.0,
            0.0,
            false,
        );

        println!("{:#?}", polygons);
    }

    #[test]
    fn test_cd_draw_curve_universal() {
        let ming = init(false);
        let mut polygons = Polygons::new();

        ming.cd_draw_curve_universal(
            &mut polygons,
            (101.07092020222045, 160.00025148691643).into(),
            (101.0, 170.0).into(),
            (101.0, 170.0).into(),
            (91.0, 170.0).into(),
            EndType::new(1.0),
            EndType::new(14.0),
            0.0,
            0.0,
            0.0,
            0.0,
        );

        println!(
            "{}",
            polygons
                .array
                .iter()
                .map(|each| {
                    let mut tmp = String::new();
                    for point in each.points() {
                        tmp.push_str(&format!("{},{} ", point.x, point.y));
                    }
                    tmp
                })
                .collect::<Vec<String>>()
                .join("\n")
        );
    }
}
