use crate::utils::{CurveSampler, Point, normalize, round};

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

#[derive(Debug)]
struct FattenResult {
    left: Vec<Point>,
    right: Vec<Point>,
}

fn generate_fatten_curve<P1, P2, P3, P4>(
    start_point: P1,
    control_point_1: P2,
    control_point_2: P3,
    end_point: P4,
    k_rate: usize,
    width_func: impl Fn(f64) -> f64,
) -> FattenResult
where
    P1: Into<Point>,
    P2: Into<Point>,
    P3: Into<Point>,
    P4: Into<Point>,
{
    let start_point = start_point.into();
    let control_point_1 = control_point_1.into();
    let control_point_2 = control_point_2.into();
    let end_point = end_point.into();

    let mut result = FattenResult {
        left: Vec::with_capacity(1000 / k_rate + 1),
        right: Vec::with_capacity(1000 / k_rate + 1),
    };
    let is_quadratic = control_point_1 == control_point_2;

    let curve_sampler = if is_quadratic {
        CurveSampler::Quadratic {
            start_point,
            control_point: control_point_1,
            end_point,
        }
    } else {
        CurveSampler::Cubic {
            start_point,
            control_point_1,
            control_point_2,
            end_point,
        }
    };

    for step in (0..=1000).step_by(k_rate) {
        let progress = step as f64 / 1000.0;

        let sampled_point = curve_sampler.sample(progress);
        let direction_vector = curve_sampler.derivative(progress);
        let width = width_func(progress);
        let normal_vector =
            if round(direction_vector.x, 8) == 0.0 && round(direction_vector.y, 8) == 0.0 {
                (-width, 0.0).into() // ???
            } else {
                normalize((-direction_vector.y, direction_vector.x), width)
            };

        result.left.push(sampled_point - normal_vector);
        result.right.push(sampled_point + normal_vector);
    }

    result
}

#[cfg(test)]
mod test {
    use crate::{curve::generate_fatten_curve, polygon::Polygon, polygons::Polygons};

    #[test]
    fn test_bold_straight_line() {
        let result = generate_fatten_curve(
            (0.0, 0.0),   // 起點 (x1, y1)
            (50.0, 0.0),  // 控制點1 (sx1, sy1) - 與控制點2相同，形成直線
            (50.0, 0.0),  // 控制點2 (sx2, sy2) - 與控制點1相同，形成直線
            (100.0, 0.0), // 終點 (x2, y2)
            50,
            |_| 10.0,
        );

        println!("{:#?}", result);
    }

    #[test]
    fn test_bold_parabola() {
        let result = generate_fatten_curve(
            (0.0, 100.0),
            (50.0, 0.0),
            (50.0, 0.0),
            (100.0, 100.0),
            25,
            |_| 10.0,
        );

        let mut points = result.left.clone();
        points.extend(result.right.iter().rev().copied());
        let mut polygons = Polygons::new();
        polygons.push(Polygon::new(points));

        println!("{}", polygons.generate_svg(true));
    }
}
