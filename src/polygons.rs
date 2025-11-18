use core::slice;

use time::{OffsetDateTime, macros::format_description};

use crate::polygon::Polygon;

#[derive(Debug)]
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

    pub fn array(&self) -> slice::Iter<'_, Polygon> {
        self.array.iter()
    }

    pub fn array_mut(&mut self) -> slice::IterMut<'_, Polygon> {
        self.array.iter_mut()
    }

    pub fn push(&mut self, mut polygon: Polygon) {
        let mut min_x = 200.0;
        let mut max_x = 0.0;
        let mut min_y = 200.0;
        let mut max_y = 0.0;

        if polygon.len() < 3 {
            return;
        }

        polygon.floor();

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

        buffer
    }

    pub fn generate_eps(&self) -> String {
        let mut buffer = format!(
            r#"%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 -208 1024 816
%%Pages: 0
%%Title: Kanji glyph
%%Creator: GlyphWiki powered by KAGE system
%%CreationDate: {}
%%EndComments
%%EndProlog"#,
            local_time()
        );
        buffer.push('\n');

        for polygon in &self.array {
            let points_arr = polygon.points();
            for j in 0..points_arr.len() {
                buffer.push_str(&format!(
                    r#"{} {} "#,
                    points_arr[j].x * 5.0,
                    1000.0 - points_arr[j].y * 5.0 - 200.0
                ));

                if j == 0 {
                    buffer.push_str(r#"newpath"#);
                    buffer.push('\n');
                    buffer.push_str(r#"moveto"#);
                    buffer.push('\n');
                } else {
                    buffer.push_str(r#"lineto"#);
                    buffer.push('\n');
                }
            }

            buffer.push_str(r#"closepath"#);
            buffer.push('\n');
            buffer.push_str(r#"fill"#);
            buffer.push('\n');
        }

        buffer.push_str(r#"%%EOF"#);
        buffer.push('\n');

        buffer
    }
}

fn local_time() -> String {
    let now = OffsetDateTime::now_local().expect("no local offset?");
    // expected format: Mon Oct 13 2025 12:34:56 GMT+0800
    let fmt = format_description!(
        "[weekday repr:short] [month repr:short] [day padding:zero] [year] \
         [hour]:[minute]:[second] GMT[offset_hour sign:mandatory][offset_minute]"
    );

    now.format(&fmt)
        .unwrap_or_else(|_| "Unknown Time".to_string())
}

#[cfg(test)]
mod tests {
    use core::f64;

    use super::*;

    #[test]
    fn test_generate_svg() {
        // case 1: simple triangle;
        let mut polygons1 = Polygons::new();
        let mut triangle = Polygon::new_empty();
        triangle.push_point((50.0, 50.0));
        triangle.push_point((150.0, 50.0));
        triangle.push_point((100.0, 150.0));
        polygons1.push(triangle);

        assert_eq!(
            polygons1.generate_svg(true),
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">
<path d="M 50,50 L 150,50 100,150 Z" fill="black" />
</svg>
"#
        );
        assert_eq!(
            polygons1.generate_svg(false),
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">
<g fill="black">
<polygon points="50,50 150,50 100,150 " />
</g>
</svg>
"#
        );

        // case 2: simple square;
        let mut polygons2 = Polygons::new();
        let mut square = Polygon::new_empty();
        square.push_point((30.0, 30.0));
        square.push_point((170.0, 30.0));
        square.push_point((170.0, 170.0));
        square.push_point((30.0, 170.0));
        polygons2.push(square);

        assert_eq!(
            polygons2.generate_svg(true),
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">
<path d="M 30,30 L 170,30 170,170 30,170 Z" fill="black" />
</svg>
"#
        );
        assert_eq!(
            polygons2.generate_svg(false),
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">
<g fill="black">
<polygon points="30,30 170,30 170,170 30,170 " />
</g>
</svg>
"#
        );
    }

    #[test]
    fn test_generate_eps() {
        // case 1: simple triangle;
        let mut polygons1 = Polygons::new();
        let mut triangle = Polygon::new_empty();
        triangle.push_point((50.0, 50.0));
        triangle.push_point((150.0, 50.0));
        triangle.push_point((100.0, 150.0));
        polygons1.push(triangle);

        assert_eq!(
            polygons1.generate_eps(),
            format!(
                "%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 -208 1024 816
%%Pages: 0
%%Title: Kanji glyph
%%Creator: GlyphWiki powered by KAGE system
%%CreationDate: {}
%%EndComments
%%EndProlog
250 550 newpath
moveto
750 550 lineto
500 50 lineto
closepath
fill
%%EOF
",
                local_time()
            )
        );

        // case 2: simple square;
        let mut polygons2 = Polygons::new();
        let mut square = Polygon::new_empty();
        square.push_point((30.0, 30.0));
        square.push_point((170.0, 30.0));
        square.push_point((170.0, 170.0));
        square.push_point((30.0, 170.0));
        polygons2.push(square);

        assert_eq!(
            polygons2.generate_eps(),
            format!(
                "%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 -208 1024 816
%%Pages: 0
%%Title: Kanji glyph
%%Creator: GlyphWiki powered by KAGE system
%%CreationDate: {}
%%EndComments
%%EndProlog
150 650 newpath
moveto
850 650 lineto
850 -50 lineto
150 -50 lineto
closepath
fill
%%EOF
",
                local_time()
            )
        );
    }

    fn complex_polygon() -> Polygons {
        let mut polygons = Polygons::new();

        // 創建一個複雜的多邊形 - 外圍矩形
        let mut outer_rect = Polygon::new_empty();
        outer_rect.push_point((20.0, 20.0));
        outer_rect.push_point((180.0, 20.0));
        outer_rect.push_point((180.0, 80.0));
        outer_rect.push_point((20.0, 80.0));
        polygons.push(outer_rect);

        // 創建一個帶有曲線的多邊形（模擬漢字筆畫）
        let mut curved_shape = Polygon::new_empty();
        curved_shape.push_point((30.0, 100.0));
        curved_shape.push_point((60.0, 100.0));
        curved_shape.push_point((75.0, 110.0, true)); // control point
        curved_shape.push_point((90.0, 130.0));
        curved_shape.push_point((90.0, 160.0));
        curved_shape.push_point((60.0, 160.0));
        curved_shape.push_point((45.0, 150.0, true)); // control point
        curved_shape.push_point((30.0, 130.0));

        polygons.push(curved_shape);

        // 創建一個圓形（使用多個控制點模擬）
        let mut circle = Polygon::new_empty();
        let center_x = 140.0;
        let center_y = 140.0;
        let radius = 30.0;
        let segments = 8;

        for i in 0..segments {
            let angle = (i as f64 / segments as f64) * 2.0 * f64::consts::PI;
            let next_angle = ((i + 1) as f64 / segments as f64) * 2.0 * f64::consts::PI;

            // 起點
            circle.push_point((
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
            ));

            // 控制點（off-curve）
            let control_angle = (angle + next_angle) / 2.0;
            let control_radius = radius * 1.2;
            circle.push_point((
                center_x + control_radius * control_angle.cos(),
                center_y + control_radius * control_angle.sin(),
                true,
            ));
        }

        polygons.push(circle);

        // 創建一個帶有旋轉變換的三角形
        let mut triangle = Polygon::new_empty();
        triangle.push_point((100.0, 40.0));
        triangle.push_point((120.0, 70.0));
        triangle.push_point((80.0, 70.0));
        triangle.rotate_90();
        triangle.translate(80.0, 20.0);

        polygons.push(triangle);

        return polygons;
    }

    #[test]
    fn test_complex_polygon() {
        let polygons = complex_polygon();

        assert_eq!(
            polygons.generate_svg(true),
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">
<path d="M 20,20 L 180,20 180,80 20,80 Z" fill="black" />
<path d="M 30,100 L 60,100 Q 75,110 90,130 L 90,160 L 60,160 Q 45,150 30,130 Z" fill="black" />
<path d="M 170,140 Q 173.2,153.7 161.2,161.2 Q 153.7,173.2 140,170 Q 126.2,173.2 118.7,161.2 Q 106.7,153.7 110,140 Q 106.7,126.2 118.7,118.7 Q 126.2,106.7 140,110 Q 153.7,106.7 161.2,118.7 Q 173.2,126.2 Z" fill="black" />
<path d="M 40,120 L 10,140 10,100 Z" fill="black" />
</svg>
"#
        );
        assert_eq!(
            polygons.generate_svg(false),
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" version="1.1" baseProfile="full" viewBox="0 0 200 200" width="200" height="200">
<g fill="black">
<polygon points="20,20 180,20 180,80 20,80 " />
<polygon points="30,100 60,100 75,110 90,130 90,160 60,160 45,150 30,130 " />
<polygon points="170,140 173.2,153.7 161.2,161.2 153.7,173.2 140,170 126.2,173.2 118.7,161.2 106.7,153.7 110,140 106.7,126.2 118.7,118.7 126.2,106.7 140,110 153.7,106.7 161.2,118.7 173.2,126.2 " />
<polygon points="40,120 10,140 10,100 " />
</g>
</svg>
"#
        );
        assert_eq!(
            polygons.generate_eps(),
            format!(
                "%!PS-Adobe-3.0 EPSF-3.0
%%BoundingBox: 0 -208 1024 816
%%Pages: 0
%%Title: Kanji glyph
%%Creator: GlyphWiki powered by KAGE system
%%CreationDate: {}
%%EndComments
%%EndProlog
100 700 newpath
moveto
900 700 lineto
900 400 lineto
100 400 lineto
closepath
fill
150 300 newpath
moveto
300 300 lineto
375 250 lineto
450 150 lineto
450 0 lineto
300 0 lineto
225 50 lineto
150 150 lineto
closepath
fill
850 100 newpath
moveto
866 31.5 lineto
806 -6 lineto
768.5 -66 lineto
700 -50 lineto
631 -66 lineto
593.5 -6 lineto
533.5 31.5 lineto
550 100 lineto
533.5 169 lineto
593.5 206.5 lineto
631 266.5 lineto
700 250 lineto
768.5 266.5 lineto
806 206.5 lineto
866 169 lineto
closepath
fill
200 200 newpath
moveto
50 100 lineto
50 300 lineto
closepath
fill
%%EOF
",
                local_time()
            )
        );
    }
}
