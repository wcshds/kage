// rust can directly use f64::hypot to calculate the hypotenuse of a right triangle

use core::f64;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Vector {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

impl Vector {
    pub(crate) fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub(crate) fn length(&self) -> f64 {
        f64::hypot(self.x, self.y)
    }

    pub(crate) fn hypot(&self) -> f64 {
        f64::hypot(self.x, self.y)
    }
}

impl Add<f64> for Vector {
    type Output = Vector;

    fn add(self, rhs: f64) -> Self::Output {
        Vector::new(self.x + rhs, self.y + rhs)
    }
}

impl Add<Vector> for f64 {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector::new(self + rhs.x, self + rhs.y)
    }
}

impl Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub<f64> for Vector {
    type Output = Vector;

    fn sub(self, rhs: f64) -> Self::Output {
        Vector::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub<Vector> for f64 {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Vector::new(self - rhs.x, self - rhs.y)
    }
}

impl Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, rhs: Vector) -> Self::Output {
        Vector::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        Vector::new(-self.x, -self.y)
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;

    fn mul(self, rhs: f64) -> Self::Output {
        Vector::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vector> for f64 {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        Vector::new(self * rhs.x, self * rhs.y)
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        let error = 1e-6;
        (self.x - other.x).abs() <= error && (self.y - other.y).abs() <= error
    }
}

impl From<(f64, f64)> for Vector {
    fn from(value: (f64, f64)) -> Self {
        Vector::new(value.0, value.1)
    }
}

impl From<[f64; 2]> for Vector {
    fn from(value: [f64; 2]) -> Self {
        Vector::new(value[0], value[1])
    }
}

impl From<Point> for Vector {
    fn from(value: Point) -> Self {
        Vector::new(value.x, value.y)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Point {
    /// The x coordinate of the point.
    pub(crate) x: f64,
    /// The y coordinate of the point.
    pub(crate) y: f64,
    /// Whether the point is an off-curve point, i.e. a control
    /// point in a quadratic Bézier curve.
    pub(crate) off_curve: Option<bool>,
}

impl Point {
    pub(crate) const INFINITY: Self = Point {
        x: f64::INFINITY,
        y: f64::INFINITY,
        off_curve: None,
    };

    pub(crate) const NEG_INFINITY: Self = Point {
        x: f64::NEG_INFINITY,
        y: f64::NEG_INFINITY,
        off_curve: None,
    };

    pub(crate) fn new(x: f64, y: f64, off_curve: Option<bool>) -> Self {
        Self { x, y, off_curve }
    }

    pub(crate) fn new_with_off_curve(x: f64, y: f64, off_curve: bool) -> Self {
        Self {
            x,
            y,
            off_curve: Some(off_curve),
        }
    }

    pub(crate) fn set_is_off_curve(self, off_curve: bool) -> Self {
        Self {
            off_curve: Some(off_curve),
            ..self
        }
    }

    pub(crate) fn min(&self, other: Point) -> Point {
        Point::new(
            self.x.min(other.x),
            self.y.min(other.y),
            if self.off_curve == other.off_curve {
                self.off_curve
            } else {
                None
            },
        )
    }

    pub(crate) fn max(&self, other: Point) -> Point {
        Point::new(
            self.x.max(other.x),
            self.y.max(other.y),
            if self.off_curve == other.off_curve {
                self.off_curve
            } else {
                None
            },
        )
    }
}

impl Add<f64> for Point {
    type Output = Point;

    fn add(self, rhs: f64) -> Self::Output {
        Point::new(self.x + rhs, self.y + rhs, self.off_curve)
    }
}

impl Add<Point> for f64 {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point::new(self + rhs.x, self + rhs.y, rhs.off_curve)
    }
}

impl Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        let off_curve = if rhs.off_curve.is_some()
            && self.off_curve.is_some()
            && rhs.off_curve.unwrap() == self.off_curve.unwrap()
        {
            rhs.off_curve
        } else {
            None
        };
        Point::new(self.x + rhs.x, self.y + rhs.y, off_curve)
    }
}

impl Add<Vector> for Point {
    type Output = Point;

    fn add(self, rhs: Vector) -> Self::Output {
        Point::new(self.x + rhs.x, self.y + rhs.y, None)
    }
}

impl Sub<f64> for Point {
    type Output = Point;

    fn sub(self, rhs: f64) -> Self::Output {
        Point::new(self.x - rhs, self.y - rhs, self.off_curve)
    }
}

impl Sub<Point> for f64 {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point::new(self - rhs.x, self - rhs.y, rhs.off_curve)
    }
}

impl Sub<Point> for Point {
    type Output = Point;

    fn sub(self, rhs: Point) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y, self.off_curve)
    }
}

impl Sub<Vector> for Point {
    type Output = Point;

    fn sub(self, rhs: Vector) -> Self::Output {
        Point::new(self.x - rhs.x, self.y - rhs.y, None)
    }
}

impl Neg for Point {
    type Output = Point;

    fn neg(self) -> Self::Output {
        Point::new(-self.x, -self.y, self.off_curve)
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, rhs: f64) -> Self::Output {
        Point::new(self.x * rhs, self.y * rhs, self.off_curve)
    }
}

impl Mul<Point> for f64 {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        Point::new(self * rhs.x, self * rhs.y, rhs.off_curve)
    }
}

impl Div<f64> for Point {
    type Output = Point;

    fn div(self, rhs: f64) -> Self::Output {
        Point::new(self.x / rhs, self.y / rhs, self.off_curve)
    }
}

impl Div<Point> for f64 {
    type Output = Point;

    fn div(self, rhs: Point) -> Self::Output {
        Point::new(self / rhs.x, self / rhs.y, rhs.off_curve)
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        let error = 1e-6;
        (self.x == other.x && self.y == other.y)
            || ((self.x - other.x).abs() <= error && (self.y - other.y).abs() <= error)
                && self.off_curve == other.off_curve
    }
}

impl From<(f64, f64)> for Point {
    /// When `off_cureve` is omitted, it is treated as an on-curve
    /// point (`off_curve`: false).
    fn from(value: (f64, f64)) -> Self {
        Point::new(value.0, value.1, Some(false))
    }
}

impl From<(f64, f64, bool)> for Point {
    fn from(value: (f64, f64, bool)) -> Self {
        Point::new(value.0, value.1, Some(value.2))
    }
}

impl From<(f64, f64, Option<bool>)> for Point {
    fn from(value: (f64, f64, Option<bool>)) -> Self {
        Point::new(value.0, value.1, value.2)
    }
}

impl From<[f64; 2]> for Point {
    fn from(value: [f64; 2]) -> Self {
        Point::new(value[0], value[1], None)
    }
}

impl From<Vector> for Point {
    fn from(value: Vector) -> Self {
        Point::new(value.x, value.y, None)
    }
}

impl From<&Point> for Point {
    fn from(value: &Point) -> Self {
        Point::new(value.x, value.y, value.off_curve)
    }
}

const EPSILON: f64 = 1e-8;

pub fn normalize<V: Into<Vector>>(vector: V, magnitude: f64) -> Vector {
    let vector: Vector = vector.into();
    if vector.x == 0.0 && vector.y == 0.0 {
        return Vector::new(magnitude.copysign(vector.x), 0.0);
    }

    let factor = magnitude / vector.length();

    Vector::new(vector.x * factor, vector.y * factor)
}

pub(crate) enum CurveSampler {
    Quadratic {
        start_point: Point,
        control_point: Point,
        end_point: Point,
    },
    Cubic {
        start_point: Point,
        control_point_1: Point,
        control_point_2: Point,
        end_point: Point,
    },
}

impl CurveSampler {
    pub(crate) fn sample(&self, progress: f64) -> Point {
        match self {
            Self::Quadratic {
                start_point,
                control_point,
                end_point,
            } => quadratic_bezier(start_point, control_point, end_point, progress),
            Self::Cubic {
                start_point,
                control_point_1,
                control_point_2,
                end_point,
            } => cubic_bezier(
                start_point,
                control_point_1,
                control_point_2,
                end_point,
                progress,
            ),
        }
    }

    pub(crate) fn derivative(&self, progress: f64) -> Vector {
        match self {
            Self::Quadratic {
                start_point,
                control_point,
                end_point,
            } => quadratic_bezier_derivative(start_point, control_point, end_point, progress),
            Self::Cubic {
                start_point,
                control_point_1,
                control_point_2,
                end_point,
            } => cubic_bezier_derivative(
                start_point,
                control_point_1,
                control_point_2,
                end_point,
                progress,
            ),
        }
    }
}

// P1: start_point, P2: control_point, P3: end_point, t: progress
// Q1 = (1 - t) * P1 + t * P2
// Q2 = (1 - t) * P2 + t * P3
// Q3 = (1 - t) * Q1 + t * Q2
//    = (1 - t) * ( (1 - t) * P1 + t * P2 ) + t * ( (1 - t) * P2 + t * P3 )
//    = (1 - t) ** 2 * P1 + 2 * t *  (1 - t) * P2 + t ** 2 * P3
pub(crate) fn quadratic_bezier<P1, P2, P3>(
    start_point: P1,
    control_point: P2,
    end_point: P3,
    progress: f64,
) -> Point
where
    P1: Into<Point>,
    P2: Into<Point>,
    P3: Into<Point>,
{
    let start_point = start_point.into();
    let control_point = control_point.into();
    let end_point = end_point.into();

    let one_minus_progress = 1.0 - progress;

    (one_minus_progress * one_minus_progress) * start_point
        + 2.0 * one_minus_progress * progress * control_point
        + (progress * progress) * end_point
}

// d/dt(quadratic_bezier) = - 2 *  (1 - t) * P1 + ( 2 * (1 - t) - 2 * t ) * P2
//                            + 2 * t * P3
//                        = 2 * (1 - t) * (-P1 + P2) + 2 * t * (-P2 + P3)
//                        = 2 * ( -P1 + P2 + t * (P1 - 2 * P2 + P3) )
/// Return d/dt(quadratic_bezier)
fn quadratic_bezier_derivative<P1, P2, P3>(
    start_point: P1,
    control_point: P2,
    end_point: P3,
    progress: f64,
) -> Vector
where
    P1: Into<Point>,
    P2: Into<Point>,
    P3: Into<Point>,
{
    let start_point = start_point.into();
    let control_point = control_point.into();
    let end_point = end_point.into();

    (2.0 * (progress * (start_point - 2.0 * control_point + end_point) - start_point
        + control_point))
        .into()
}

// P1: start_point, P2: control_point, P3: end_point, t: progress
// Q1 = (1 - t) * P1 + t * P2
// Q2 = (1 - t) * P2 + t * P3
// Q3 = (1 - t) * P3 + t * P4
// Q4 = (1 - t) * Q1 + t * Q2
//    = (1 - t) * ( (1 - t) * P1 + t * P2 ) + t * ( (1 - t) * P2 + t * P3 )
//    = (1 - t) ** 2 * P1 + 2 * t *  (1 - t) * P2 + t ** 2 * P3
// Q5 = (1 - t) * Q2 + t * Q3
//    = (1 - t) * ( (1 - t) * P2 + t * P3 ) + t * ( (1 - t) * P3 + t * P4 )
//    = (1 - t) ** 2 * P2 + 2 * t *  (1 - t) * P3 + t ** 2 * P4
// Q6 = (1 - t) * Q4 + t * Q5
//    = (1 - t) ** 3 * P1 + 2 * t * (1 - t) ** 2 * P2 + t ** 2 * (1 - t) * P3
//         + t * (1 - t) ** 2 * P2 + 2 * t ** 2 * (1 - t) * P3 + t ** 3 * P4
//    = (1 - t) ** 3 * P1 + 3 * t * (1 - t) ** 2 * P2
//         + 3 * t ** 2 * (1 - t) * P3 + t ** 3 * P4
pub(crate) fn cubic_bezier<P1, P2, P3, P4>(
    start_point: P1,
    control_point_1: P2,
    control_point_2: P3,
    end_point: P4,
    progress: f64,
) -> Point
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

    let one_minus_progress = 1.0 - progress;

    one_minus_progress.powi(3) * start_point
        + 3.0 * progress * one_minus_progress.powi(2) * control_point_1
        + 3.0 * progress.powi(2) * one_minus_progress * control_point_2
        + progress.powi(3) * end_point
}

fn cubic_bezier_derivative<P1, P2, P3, P4>(
    start_point: P1,
    control_point_1: P2,
    control_point_2: P3,
    end_point: P4,
    progress: f64,
) -> Vector
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

    (3.0 * (progress
        * (progress * (-start_point + 3.0 * control_point_1 - 3.0 * control_point_2 + end_point)
            + 2.0 * (start_point - 2.0 * control_point_1 + control_point_2))
        - start_point
        + control_point_1))
        .into()
}

fn ternary_search_min(func: impl Fn(f64) -> f64, left: f64, right: f64, epsilon: f64) -> f64 {
    let mut left = left;
    let mut right = right;

    while (right - left) > epsilon {
        let x1 = left + (right - left) / 3.0;
        let x2 = right - (right - left) / 3.0;
        let y1 = func(x1);
        let y2 = func(x2);
        if y1 < y2 {
            right = x2;
        } else {
            left = x1;
        }
    }

    left + (right - left) / 2.0
}

fn ternary_search_max(func: impl Fn(f64) -> f64, left: f64, right: f64, epsilon: f64) -> f64 {
    ternary_search_min(|x: f64| -func(x), left, right, epsilon)
}

pub(crate) fn round(num: f64, decimals: i32) -> f64 {
    let factor = 10.0f64.powi(decimals);
    (num * factor).round() / factor
}

pub(crate) fn is_quadratic<P1, P2>(control_point_1: P1, control_point_2: P2) -> bool
where
    P1: Into<Point>,
    P2: Into<Point>,
{
    let control_point_1: Point = control_point_1.into();
    let control_point_2: Point = control_point_2.into();

    (control_point_1.x - control_point_2.x).abs() <= EPSILON
        && (control_point_1.y - control_point_2.y).abs() <= EPSILON
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quadratic_bezier() {
        let start_point = (-6.0, 5.0);
        let control_point = (-1.2, 0.5);
        let end_point = (4.0, 8.0);
        let progress = 0.25;

        let point = quadratic_bezier(start_point, control_point, end_point, progress);

        assert_eq!(point, (-3.575, 3.5).into());
    }

    #[test]
    fn test_quadratic_bezier_derivative() {
        // quadraticBezierDeriv(5, -2, 7, 0.3)
        let start_point = (5.0, -3.0);
        let control_point = (-2.0, 2.0);
        let end_point = (7.0, 5.0);
        let progress = 0.3;

        let point = quadratic_bezier_derivative(start_point, control_point, end_point, progress);

        assert_eq!(point, (-4.4, 8.8).into());
    }

    #[test]
    fn test_cubic_bezier() {
        // ,  1.2, 2.2
        let start_point = (3.2, -9.2);
        let control_point_1 = (-5.3, 5.8);
        let control_point_2 = (4.2, 1.2);
        let end_point = (8.2, 2.2);
        let progress = 0.3;

        let point = cubic_bezier(
            start_point,
            control_point_1,
            control_point_2,
            end_point,
            progress,
        );

        assert_eq!(point, (-0.22449999999999976, -0.31159999999999943).into());
    }

    #[test]
    fn test_cubic_bezier_derivative() {
        let start_point = (3.2, -9.2);
        let control_point_1 = (-5.3, 5.8);
        let control_point_2 = (4.2, 1.2);
        let end_point = (8.2, 2.2);
        let progress = 0.3;

        let point = cubic_bezier_derivative(
            start_point,
            control_point_1,
            control_point_2,
            end_point,
            progress,
        );

        assert_eq!(point, (0.5549999999999962, 16.524).into());
    }

    #[test]
    fn test_ternary_search_min() {
        let func = |x: f64| (x + 1.0).powi(2);
        let left = -2.0;
        let right = 5.0;
        let epsilon = 1e-5;

        let result = ternary_search_min(func, left, right, epsilon);

        // -0.999999849748316
        assert!(result - (-1.0) <= 1e-5);
    }

    #[test]
    fn test_round() {
        let num = 1.23456789;
        let result = round(num, 2);
        assert_eq!(result, 1.23);
    }
}
