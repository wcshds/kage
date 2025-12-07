use crate::utils::{CurveSampler, Point, is_quadratic, normalize, round};

const INITIAL_RATE: f64 = 0.5;

#[derive(Debug)]
pub(crate) struct SplitResult {
    pub(crate) index: usize,
    pub(crate) segments: [[Point; 3]; 2],
}

// FIXME: may be redundant
pub(crate) fn split_quadratic_bezier_curve(
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
pub(crate) struct FattenResult {
    pub(crate) left: Vec<Point>,
    pub(crate) right: Vec<Point>,
}

// FIXME: add more tests
pub(crate) fn generate_fatten_curve<P1, P2, P3, P4>(
    start_point: P1,
    control_point_1: P2,
    control_point_2: P3,
    end_point: P4,
    sample_step: usize,
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
        left: Vec::with_capacity(1000 / sample_step + 1),
        right: Vec::with_capacity(1000 / sample_step + 1),
    };

    let curve_sampler = if is_quadratic(control_point_1, control_point_2) {
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

    for step in (0..=1000).step_by(sample_step) {
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

#[derive(Debug)]
pub(crate) struct QuadraticBezierFitResult {
    pub(crate) start_point: Point,
    pub(crate) control_point: Point,
    pub(crate) end_point: Point,
}

pub(crate) fn fit_quadratic_bezier(points: &[Point]) -> Option<QuadraticBezierFitResult> {
    if points.len() == 2 {
        let temp = (points[0] + points[1]) * 0.5;
        return Some(QuadraticBezierFitResult {
            start_point: points[0],
            control_point: (temp.x, temp.y, true).into(),
            end_point: points[1],
        });
    } else if points.len() <= 1 {
        return None;
    }

    let start_point = points[0];
    let end_point = points[points.len() - 1];

    let mut numerator: Point = (0.0, 0.0).into();
    let mut denominator = 0.0;
    for (step, point) in (1..(points.len() - 1)).zip(points.iter().skip(1)) {
        let progress = step as f64 / (points.len() - 1) as f64;
        let remain = 1.0 - progress;
        let remain_squared = remain * remain;
        let progress_squared = progress * progress;
        let progress_times_remain = progress * remain;

        numerator = numerator
            + progress_times_remain
                * (point.clone() - remain_squared * start_point - progress_squared * end_point);
        denominator += 2.0 * progress_times_remain * progress_times_remain;
    }

    if denominator == 0.0 {
        return None;
    }

    let control_point = numerator / denominator;
    Some(QuadraticBezierFitResult {
        start_point,
        control_point: (control_point.x, control_point.y, true).into(),
        end_point,
    })
}

#[cfg(test)]
mod test {
    use crate::{
        curve::{fit_quadratic_bezier, generate_fatten_curve},
        polygon::Polygon,
        polygons::Polygons,
    };

    #[test]
    fn test_bold_straight_line() {
        let result = generate_fatten_curve(
            (0.0, 0.0),
            (50.0, 0.0),
            (50.0, 0.0),
            (100.0, 0.0),
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
        polygons.push(Polygon::new(points, None));

        println!("{}", polygons.generate_svg(true));
    }

    #[test]
    fn test_fit_parabola() {
        let result = fit_quadratic_bezier(&vec![
            (0.0, 0.0).into(),
            (1.0, 1.0).into(),
            (2.0, 4.0).into(),
            (3.0, 9.0).into(),
            (4.0, 16.0).into(),
        ]);

        println!("{:#?}", result);
    }

    #[test]
    fn test_fit_arc() {
        let result = fit_quadratic_bezier(&vec![
            [0.0, 0.0].into(),
            [0.5, 0.866].into(),
            [1.0, 1.732].into(),
            [1.5, 2.598].into(),
            [2.0, 3.464].into(),
        ]);

        println!("{:#?}", result);
    }
}
