use crate::utils::Point;

const PRECISION: f64 = 10.0;

#[derive(Debug, Clone)]
pub struct Polygon {
    points: Vec<Point>,
}

impl Polygon {
    pub fn new<P: Into<Point>>(points: Vec<P>) -> Self {
        let mut new_points = Vec::with_capacity(points.len());
        for point in points {
            let raw_point: Point = point.into();
            new_points.push(Self::create_internal_point(
                raw_point.x,
                raw_point.y,
                raw_point.off_curve,
            ));
        }
        Self { points: new_points }
    }

    /// Return a `Polygon` with the given length, the points are
    /// all initialized to the origin (0.0, 0.0, false).
    pub fn new_with_length(length: usize) -> Self {
        let points = vec![(0.0, 0.0, false).into(); length];

        Self { points }
    }

    pub fn new_empty() -> Self {
        let points = Vec::new();
        Self { points }
    }

    pub fn new_empty_with_capacity(capacity: usize) -> Self {
        let points = Vec::with_capacity(capacity);
        Self { points }
    }

    pub fn points(&self) -> Vec<Point> {
        let mut new_points = Vec::with_capacity(self.points.len());
        for point in &self.points {
            new_points.push(Point::new(
                point.x / PRECISION,
                point.y / PRECISION,
                point.off_curve,
            ));
        }
        new_points
    }

    pub fn len(&self) -> usize {
        self.points.len()
    }

    pub fn push(&mut self, x: f64, y: f64, off: Option<bool>) {
        let point = Self::create_internal_point(x, y, off);
        self.points.push(point);
    }

    pub fn push_point<P: Into<Point>>(&mut self, point: P) {
        let raw_point = point.into();
        self.push(raw_point.x, raw_point.y, raw_point.off_curve);
    }

    pub fn get(&self, index: usize) -> Option<Point> {
        if index >= self.points.len() {
            return None;
        }
        let raw_point = unsafe { self.points.get_unchecked(index) };
        Some(Point::new(
            raw_point.x / PRECISION,
            raw_point.y / PRECISION,
            raw_point.off_curve,
        ))
    }

    pub fn set(&mut self, index: usize, x: f64, y: f64, off: Option<bool>) -> Result<(), String> {
        if index >= self.points.len() {
            return Err("Index out of bounds.".to_string());
        }

        let point = Self::create_internal_point(x, y, off);
        self.points[index] = point;

        Ok(())
    }

    pub fn set_point<P: Into<Point>>(&mut self, index: usize, point: P) -> Result<(), String> {
        if index >= self.points.len() {
            return Err("Index out of bounds.".to_string());
        }
        let raw_point: Point = point.into();

        self.set(index, raw_point.x, raw_point.y, raw_point.off_curve)
    }

    // FIXME: may be add a flag to improve the performance
    pub fn reverse(&mut self) {
        self.points.reverse();
    }

    pub fn concat(&mut self, other: Polygon) {
        self.points.extend(other.points);
    }

    /// Removes the first point from its contour. Does nothing if the contour is empty.
    pub fn shift(&mut self) {
        if self.points.is_empty() {
            return;
        }

        // FIXME: Consider using `VecDeque` to improve the performance.
        self.points.remove(0);
    }

    /// Inserts a new point at the beginning of its contour.
    pub fn unshift(&mut self, x: f64, y: f64, off: Option<bool>) {
        let point = Self::create_internal_point(x, y, off);
        self.points.insert(0, point);
    }

    pub fn unshift_point(&mut self, point: Point) {
        self.unshift(point.x, point.y, point.off_curve);
    }

    /// Translates the whole polygon by the given amount.
    ///
    /// # Arguments
    ///
    /// * `dx` - The x-amount of translation.
    /// * `dy` - The y-amount of translation.
    ///
    /// # Returns
    ///
    /// A mutable reference to the translated polygon (for chaining).
    pub fn translate(&mut self, dx: f64, dy: f64) -> &mut Self {
        let dx = dx * PRECISION;
        let dy = dy * PRECISION;

        for point in self.points.iter_mut() {
            point.x += dx;
            point.y += dy;
        }

        self
    }

    /// Flips the sign of the x-coordinate of each point in the contour.
    ///
    /// # Returns
    ///
    /// A mutable reference to the reflected polygon (for chaining).
    pub fn reflect_x(&mut self) -> &mut Self {
        for point in self.points.iter_mut() {
            point.x = -point.x;
        }

        self
    }

    /// Flips the sign of the y-coordinate of each point in the contour.
    ///
    /// # Returns
    ///
    /// A mutable reference to the reflected polygon (for chaining).
    pub fn reflect_y(&mut self) -> &mut Self {
        for point in self.points.iter_mut() {
            point.y = -point.y;
        }

        self
    }

    /// Rotates the whole polygon by 90 degrees clockwise.
    ///
    /// # Returns
    ///
    /// A mutable reference to the rotated polygon (for chaining).
    pub fn rotate_90(&mut self) -> &mut Self {
        for point in self.points.iter_mut() {
            (point.x, point.y) = (-point.y, point.x);
        }

        self
    }

    /// Rotates the whole polygon by 180 degrees clockwise.
    ///
    /// # Returns
    ///
    /// A mutable reference to the rotated polygon (for chaining).
    pub fn rotate_180(&mut self) -> &mut Self {
        for point in self.points.iter_mut() {
            (point.x, point.y) = (-point.x, -point.y);
        }

        self
    }

    /// Rotates the whole polygon by 270 degrees clockwise.
    ///
    /// # Returns
    ///
    /// A mutable reference to the rotated polygon (for chaining).
    pub fn rotate_270(&mut self) -> &mut Self {
        for point in self.points.iter_mut() {
            (point.x, point.y) = (point.y, -point.x);
        }

        self
    }

    pub fn floor(&mut self) -> &mut Self {
        for point in self.points.iter_mut() {
            point.x = point.x.floor();
            point.y = point.y.floor();
        }

        self
    }

    fn create_internal_point(x: f64, y: f64, off: Option<bool>) -> Point {
        Point::new(x * PRECISION, y * PRECISION, off)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_construction() {
        let polygon_len3 = Polygon::new_with_length(3);
        assert_eq!(polygon_len3.len(), 3);
        for i in 0..3 {
            assert_eq!(polygon_len3.get(i), Some(Point::new(0.0, 0.0, Some(false))));
        }

        let points: Vec<Point> = vec![
            (1.0, 2.0, false).into(),
            (3.0, 4.0, true).into(),
            (5.0, 6.0).into(), // off_curve is default to false
        ];
        let polygon_from_points = Polygon::new(points);
        assert_eq!(polygon_len3.len(), 3);
        let point_0 = polygon_from_points.get(0);
        assert_eq!(point_0, Some(Point::new(1.0, 2.0, Some(false))));
        let point_1 = polygon_from_points.get(1);
        assert_eq!(point_1, Some(Point::new(3.0, 4.0, Some(true))));
        let point_2 = polygon_from_points.get(2);
        assert_eq!(point_2, Some(Point::new(5.0, 6.0, Some(false))));
    }

    #[test]
    fn test_polygon_basic_operations() {
        let mut polygon = Polygon::new_empty();

        polygon.push(10.0, 20.0, Some(false));
        polygon.push_point((3.0, 4.0, true));
        assert_eq!(polygon.len(), 2);

        let point_0 = polygon.get(0);
        assert_eq!(point_0, Some(Point::new(10.0, 20.0, Some(false))));
        let point_1 = polygon.get(1);
        assert_eq!(point_1, Some(Point::new(3.0, 4.0, Some(true))));

        polygon.set_point(0, (50.0, 60.0, true)).unwrap();
        assert_eq!(polygon.get(0), Some(Point::new(50.0, 60.0, Some(true))));

        polygon.push_point((70.0, 80.0, false));
        assert_eq!(polygon.len(), 3);

        let point_2 = polygon.get(2);
        assert_eq!(point_2, Some(Point::new(70.0, 80.0, Some(false))));

        // When `off_cureve` is omitted, it is treated as an on-curve
        // point (`off_curve`: false). See Point::from((f64, f64)).
        polygon.set_point(2, (90.0, 100.0)).unwrap();
        assert_eq!(polygon.get(2), Some(Point::new(90.0, 100.0, Some(false))));
    }

    #[test]
    fn test_polygon_array_access() {
        let points = vec![(1.0, 2.0, false), (3.0, 4.0, true), (5.0, 6.0, false)];
        let polygon = Polygon::new(points);
        assert_eq!(polygon.get(0), Some(Point::new(1.0, 2.0, Some(false))));
        assert_eq!(polygon.get(1), Some(Point::new(3.0, 4.0, Some(true))));
        assert_eq!(polygon.get(2), Some(Point::new(5.0, 6.0, Some(false))));

        let array = polygon.points();
        assert_eq!(
            array,
            vec![
                (1.0, 2.0, false).into(),
                (3.0, 4.0, true).into(),
                (5.0, 6.0, false).into(),
            ]
        );
    }

    #[test]
    fn test_polygon_modifications() {
        let mut polygon =
            Polygon::new(vec![(1.0, 2.0, false), (3.0, 4.0, true), (5.0, 6.0, false)]);

        polygon.reverse();
        assert_eq!(polygon.get(0), Some(Point::new(5.0, 6.0, Some(false))));
        assert_eq!(polygon.get(1), Some(Point::new(3.0, 4.0, Some(true))));
        assert_eq!(polygon.get(2), Some(Point::new(1.0, 2.0, Some(false))));

        polygon.shift();
        assert_eq!(polygon.len(), 2);
        assert_eq!(polygon.get(0), Some(Point::new(3.0, 4.0, Some(true))));
        assert_eq!(polygon.get(1), Some(Point::new(1.0, 2.0, Some(false))));

        polygon.unshift(7.0, 8.0, Some(true));
        assert_eq!(polygon.len(), 3);
        assert_eq!(polygon.get(0), Some(Point::new(7.0, 8.0, Some(true))));
        assert_eq!(polygon.get(1), Some(Point::new(3.0, 4.0, Some(true))));
        assert_eq!(polygon.get(2), Some(Point::new(1.0, 2.0, Some(false))));
    }

    #[test]
    fn test_polygon_concat() {
        let mut polygon_1 = Polygon::new(vec![(1.0, 2.0, false), (3.0, 4.0, true)]);
        let polygon_2 = Polygon::new(vec![(5.0, 6.0, false), (7.0, 8.0, true)]);

        polygon_1.concat(polygon_2.clone());
        assert_eq!(polygon_1.len(), 4);

        assert_eq!(polygon_1.get(0), Some(Point::new(1.0, 2.0, Some(false))));
        assert_eq!(polygon_1.get(1), Some(Point::new(3.0, 4.0, Some(true))));
        assert_eq!(polygon_1.get(2), Some(Point::new(5.0, 6.0, Some(false))));
        assert_eq!(polygon_1.get(3), Some(Point::new(7.0, 8.0, Some(true))));

        assert_eq!(polygon_2.len(), 2);
    }

    #[test]
    fn test_polygon_transformations() {
        let mut polygon_1 =
            Polygon::new(vec![(1.0, 2.0, false), (3.0, 4.0, true), (5.0, 6.0, false)]);

        polygon_1.translate(10.0, 20.0);
        assert_eq!(polygon_1.get(0), Some(Point::new(11.0, 22.0, Some(false))));
        assert_eq!(polygon_1.get(1), Some(Point::new(13.0, 24.0, Some(true))));
        assert_eq!(polygon_1.get(2), Some(Point::new(15.0, 26.0, Some(false))));

        polygon_1.reflect_x();
        assert_eq!(polygon_1.get(0), Some(Point::new(-11.0, 22.0, Some(false))));
        assert_eq!(polygon_1.get(1), Some(Point::new(-13.0, 24.0, Some(true))));
        assert_eq!(polygon_1.get(2), Some(Point::new(-15.0, 26.0, Some(false))));

        polygon_1.reflect_y();
        assert_eq!(
            polygon_1.get(0),
            Some(Point::new(-11.0, -22.0, Some(false)))
        );
        assert_eq!(polygon_1.get(1), Some(Point::new(-13.0, -24.0, Some(true))));
        assert_eq!(
            polygon_1.get(2),
            Some(Point::new(-15.0, -26.0, Some(false)))
        );

        let mut polygon_2 = Polygon::new(vec![(1.0, 0.0, false), (0.0, 1.0, false)]);
        polygon_2.rotate_90();
        assert_eq!(polygon_2.get(0), Some(Point::new(-0.0, 1.0, Some(false))));
        assert_eq!(polygon_2.get(1), Some(Point::new(-1.0, 0.0, Some(false))));

        polygon_2.rotate_180();
        assert_eq!(polygon_2.get(0), Some(Point::new(0.0, -1.0, Some(false))));
        assert_eq!(polygon_2.get(1), Some(Point::new(1.0, -0.0, Some(false))));

        polygon_2.rotate_270();
        assert_eq!(polygon_2.get(0), Some(Point::new(-1.0, -0.0, Some(false))));
        assert_eq!(polygon_2.get(1), Some(Point::new(-0.0, -1.0, Some(false))));
    }

    #[test]
    fn test_polygon_floor() {
        let mut polygon = Polygon::new(vec![(1.7, 2.3, false), (3.1, 4.9, true)]);

        polygon.floor();

        assert_eq!(polygon.len(), 2);
        assert_eq!(polygon.get(0), Some(Point::new(1.7, 2.3, Some(false))));
        assert_eq!(polygon.get(1), Some(Point::new(3.1, 4.9, Some(true))));
    }
}
