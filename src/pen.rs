use crate::{
    polygon::Polygon,
    utils::{Point, Vector, normalize},
};

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
        let Vector { x: dx, y: dy } = normalize((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(-dx, -dy)
    }

    pub fn set_right(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } = normalize((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(dx, dy)
    }

    pub fn set_up(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } = normalize((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(-dy, dx)
    }

    pub fn set_down(&mut self, other_x: f64, other_y: f64) -> &mut Self {
        let other_point: Point = (other_x, other_y).into();
        let Vector { x: dx, y: dy } = normalize((other_point - self.global_point).into(), 1.0);

        self.set_matrix2(dy, -dx)
    }

    pub fn get_point(&self, local_x: f64, local_y: f64, off: bool) -> Point {
        let new_x = self.global_point.x + self.cos_theta * local_x + -self.sin_theta * local_y;
        let new_y = self.global_point.y + self.sin_theta * local_x + self.cos_theta * local_y;

        Point::new(new_x, new_y, Some(off))
    }

    pub fn r#move(&mut self, local_dx: f64, local_dy: f64) -> &mut Self {
        self.global_point = self.get_point(local_dx, local_dy, false);

        self
    }

    pub fn get_polygon<P: AsRef<Point>>(&self, local_points: &[P]) -> Polygon {
        let local_points = local_points
            .iter()
            .map(|p| {
                let local_point = p.as_ref();
                self.get_point(
                    local_point.x,
                    local_point.y,
                    local_point.off_curve.unwrap_or(false),
                )
            })
            .collect::<Vec<Point>>();

        Polygon::new(local_points)
    }
}
