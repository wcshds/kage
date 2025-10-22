use crate::utils::Point;

const INITIAL_RATE: f64 = 0.5;

struct SplitResult {
    index: usize,
    segments: [[Point; 3]; 2],
}

// FIXME: may be redundant
fn split_quadratic_bezier_curve(
    start_point: Point,
    control_point: Point,
    end_point: Point,
    sampled_points: &[Point],
) -> SplitResult {
    let split_index = (sampled_points.len() as f64 * INITIAL_RATE).floor() as usize;
    let actual_rate = split_index as f64 / sampled_points.len() as f64;

    let new_control_point_1 = (1.0 - actual_rate) * start_point + actual_rate * control_point;
    let new_control_point_2 = (1.0 - actual_rate) * control_point + actual_rate * end_point;
    let intermediate_point_on_curve =
        (1.0 - actual_rate) * new_control_point_1 + actual_rate * new_control_point_2;

    SplitResult {
        index: split_index,
        segments: [
            [
                start_point,
                new_control_point_1,
                intermediate_point_on_curve,
            ],
            [intermediate_point_on_curve, new_control_point_2, end_point],
        ],
    }
}
