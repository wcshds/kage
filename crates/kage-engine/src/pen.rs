use crate::{
    polygon::Polygon,
    utils::{Point, Rgb, Vector, normalize},
};

#[derive(Debug)]
pub struct Pen {
    global_point: Point,
    cos_theta: f64,
    sin_theta: f64,
}

impl Pen {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            global_point: (x, y).into(),
            cos_theta: 1.0,
            sin_theta: 0.0,
        }
    }

    pub fn set_matrix2(&mut self, cos_theta: f64, sin_theta: f64) -> &mut Self {
        self.cos_theta = cos_theta;
        self.sin_theta = sin_theta;

        self
    }

    pub fn set_left(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } =
            normalize::<Vector>((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(-dx, -dy)
    }

    pub fn set_right(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } =
            normalize::<Vector>((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(dx, dy)
    }

    pub fn set_up(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } =
            normalize::<Vector>((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(-dy, dx)
    }

    pub fn set_down(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } =
            normalize::<Vector>((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(dy, -dx)
    }

    pub fn get_point(&self, local_x: f64, local_y: f64, off: bool) -> Point {
        let new_x = self.global_point.x + self.cos_theta * local_x + -self.sin_theta * local_y;
        let new_y = self.global_point.y + self.sin_theta * local_x + self.cos_theta * local_y;

        Point::new(new_x, new_y, Some(off))
    }

    pub fn move_local(&mut self, local_dx: f64, local_dy: f64) -> &mut Self {
        self.global_point = self.get_point(local_dx, local_dy, false);

        self
    }

    pub fn get_polygon<P>(&self, local_points: &[P], color: Option<Rgb>) -> Polygon
    where
        P: Into<Point> + Copy,
    {
        let local_points = local_points
            .iter()
            .map(|local_point| {
                let local_point: Point = (*local_point).into();
                self.get_point(
                    local_point.x,
                    local_point.y,
                    local_point.off_curve.unwrap_or(false),
                )
            })
            .collect::<Vec<Point>>();

        Polygon::new(local_points, color)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pen_construction() {
        let pen = Pen::new(100.0, 100.0);
        let point = pen.get_point(10.0, 0.0, false);

        assert_eq!(point, (110.0, 100.0).into());
    }

    #[test]
    fn test_up() {
        let mut pen = Pen::new(3.0, 2.0);
        pen.set_up(6.0, 1.0);
        let point3 = pen.get_point(4.0, -5.0, false);

        assert_eq!(point3, (9.008327554319921, 4.213594362117865).into());
    }

    #[test]
    fn test_right() {
        let mut pen = Pen::new(3.0, 2.0);
        pen.set_right(6.0, 1.0);
        let point = pen.get_point(4.0, -5.0, false);

        assert_eq!(point, (5.213594362117865, -4.008327554319921).into());
    }

    #[test]
    fn test_down() {
        let mut pen = Pen::new(3.0, 2.0);
        pen.set_down(6.0, 1.0);
        let point = pen.get_point(4.0, -5.0, false);

        assert_eq!(point, (-3.008327554319921, -0.2135943621178653).into());
    }

    #[test]
    fn test_left() {
        let mut pen = Pen::new(3.0, 2.0);
        pen.set_left(6.0, 1.0);
        let point = pen.get_point(4.0, -5.0, false);

        assert_eq!(point, (0.7864056378821347, 8.008327554319921).into());
    }

    #[test]
    fn test_move_local() {
        let mut pen = Pen::new(3.0, 2.0);
        pen.set_left(6.0, 1.0);
        pen.move_local(12.0, -3.0);
        let point = pen.get_point(4.0, 7.0, false);

        assert_eq!(point, (-13.44384383287557, 3.264911064067353).into());
    }

    #[test]
    fn test_get_polygon() {
        let mut pen = Pen::new(3.0, 2.0);
        pen.set_left(6.0, 1.0);

        let local_points: Vec<Point> = vec![
            (0.0, 0.0, false).into(),
            (10.0, 0.0, false).into(),
            (10.0, 10.0, false).into(),
            (0.0, 10.0, false).into(),
        ];

        let polygon = pen.get_polygon(&local_points, None);

        assert_eq!(
            polygon.points(),
            vec![
                (3.0, 2.0, false).into(),
                (-6.486832980505139, 5.16227766016838, false).into(),
                (-9.649110640673518, -4.324555320336758, false).into(),
                (-0.16227766016837952, -7.486832980505139, false).into(),
            ]
        );
    }
}
