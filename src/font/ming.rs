use crate::{
    curve::{
        FattenResult, SplitResult, fit_quadratic_bezier, generate_fatten_curve,
        split_quadratic_bezier_curve,
    },
    polygon::Polygon,
    polygons::Polygons,
    stroke::{EndKind, EndType},
    utils::{Point, Vector, is_quadratic},
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
                            / (self.k_min_width_vertical - width_change_rate / 2.0);
                        let width_slope = (start_width_reduction / 2.0)
                            / (self.k_min_width_vertical - width_change_rate);

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
                    self.k_min_width_vertical * width_delta
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

                    self.k_min_width_vertical * width_delta
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
}

#[cfg(test)]
mod test {
    use crate::{font::ming::Ming, polygons::Polygons, stroke::EndType};

    fn init() -> Ming {
        Ming {
            k_rate: 100,
            k_min_width_horizontal: 2.0,
            k_min_width_triangle: 2.0,
            k_min_width_vertical: 6.0,
            k_width: 5.0,
            k_square_terminal: 3.0,
            k_l2rdfatten: 1.1,
            k_mage: 10.0,
            k_use_curve: true,
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
        let ming = init();
        let mut polygons = Polygons::new();

        ming.draw_curve_body(
            &mut polygons,
            (151.0, 23.0).into(),
            (146.0, 178.0).into(),
            (146.0, 178.0).into(),
            (182.0, 182.0).into(),
            EndType::new(22.0),
            EndType::new(15.0),
            0.0,
            0.0,
        );

        println!("{}", polygons.generate_svg(true));
    }
}
