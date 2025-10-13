use crate::polygon::Polygon;

pub struct Polygons {
    array: Vec<Polygon>,
}

impl Polygons {
    pub fn new() -> Self {
        Self { array: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.array.clear();
    }

    pub fn push(&mut self, polygon: Polygon) {
        let mut min_x = 200.0;
        let mut max_x = 0.0;
        let mut min_y = 200.0;
        let mut max_y = 0.0;

        for point in polygon.points() {
            if point.x < min_x {
                min_x = point.x;
            }
            if point.x > max_x {
                max_x = point.x;
            }
            if point.y < min_y {
                min_y = point.y;
            }
            if point.y > max_y {
                max_y = point.y;
            }
            if point.x.is_nan() || point.y.is_nan() {
                return;
            }
        }

        if min_x != max_x && min_y != max_y {
            self.array.push(polygon);
        }
    }

    pub fn generate_svg(&self, curve: bool) -> String {
        let mut buffer = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">"#.to_string();
        buffer.push('\n');

        if curve {
            for polygon in &self.array {
                let points_arr = polygon.points();
                let mut mode = "L";
                buffer.push_str(r#"<path d=""#);
                for j in 0..points_arr.len() {
                    if j == 0 {
                        buffer.push_str(r#"M "#);
                    } else if points_arr[j].off_curve.is_some_and(|off| off) {
                        buffer.push_str(r#"Q "#);
                        mode = "Q";
                    } else if mode == "Q"
                        && (points_arr[j - 1].off_curve.is_none()
                            || !points_arr[j - 1].off_curve.unwrap())
                    {
                        buffer += "L ";
                    } else if mode == "L" && j == 1 {
                        buffer += "L ";
                    }
                    buffer.push_str(&format!(r#"{},{} "#, points_arr[j].x, points_arr[j].y));
                }
                buffer.push_str(r#"Z" fill="black" />"#);
                buffer.push('\n');
            }
        } else {
            buffer.push_str(r#"<g fill="black">"#);
            buffer.push('\n');
            buffer.push_str(
                &self
                    .array
                    .iter()
                    .map(|points_arr| {
                        let mut tmp = format!(
                            r#"<polygon points="{}" />"#,
                            points_arr
                                .points()
                                .iter()
                                .map(|point| format!(r#"{},{} "#, point.x, point.y))
                                .collect::<Vec<String>>()
                                .join("")
                        );
                        tmp.push('\n');

                        tmp
                    })
                    .collect::<Vec<String>>()
                    .join(""),
            );
            buffer.push_str(r#"</g>"#);
            buffer.push('\n');
        }

        buffer.push_str(r#"</svg>"#);
        buffer.push('\n');

        return buffer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_svg() {
        // console.log("=== 測試 1: 簡單三角形 ===");
        // const polygons1 = new Polygons();
        // const triangle = new Polygon();
        // triangle.push(50, 50);
        // triangle.push(150, 50);
        // triangle.push(100, 150);
        // polygons1.push(triangle);
        // console.log(polygons1.generateSVG());
        let mut polygons1 = Polygons::new();
        let mut triangle = Polygon::new_empty();
        triangle.push_point((50.0, 50.0));
        triangle.push_point((150.0, 50.0));
        triangle.push_point((100.0, 150.0));
        polygons1.push(triangle);

        println!("{}", polygons1.generate_svg(true));
    }
}
