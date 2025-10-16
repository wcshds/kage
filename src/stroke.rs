use crate::{two_d, utils::Point};

fn stretch_numeric(dest_pivot: f64, src_pivot: f64, origin_point: f64, min: f64, max: f64) -> f64 {
    let (p1, p2, p3, p4) = if origin_point < src_pivot + 100.0 {
        (min, src_pivot + 100.0, min, dest_pivot + 100.0)
    } else {
        (src_pivot + 100.0, max, dest_pivot + 100.0, max)
    };

    ((origin_point - p1) / (p2 - p1) * (p4 - p3) + p3).floor()
}

fn stretch<P1, P2, P3, P4, P5>(
    dest_pivot: P1,
    src_pivot: P2,
    origin_point: P3,
    min: P4,
    max: P5,
) -> Point
where
    P1: Into<Point>,
    P2: Into<Point>,
    P3: Into<Point>,
    P4: Into<Point>,
    P5: Into<Point>,
{
    let dest_pivot: Point = dest_pivot.into();
    let src_pivot: Point = src_pivot.into();
    let origin_point: Point = origin_point.into();
    let min: Point = min.into();
    let max: Point = max.into();

    let x = stretch_numeric(dest_pivot.x, src_pivot.x, origin_point.x, min.x, max.x);
    let y = stretch_numeric(dest_pivot.y, src_pivot.y, origin_point.y, min.y, max.y);

    Point::new(x, y, origin_point.off_curve)
}

#[repr(u32)]
enum StrokeType {
    // 1 ~ 6: Stroke Lines
    /// 直線
    StraightLine = 1, // 2 control point
    /// 曲線
    Curve = 2, // 3 control points
    /// 折線
    Polyline = 3, // 3 control points
    /// 「乙」形線
    OtsuLine = 4, // 3 control points
    /// 複曲線
    CompoundCurve = 6, // 4 control points
    /// 豎撇
    VerticalSweep = 7, // 4 control points

    /// 未知的筆劃類型
    Unknown(u32),
    // // 99: component reference line
    // /// 部件引用行
    // ComponentReferenceLine = 99,

    // // 0: Special Lines
    // /// 特殊行
    // SpecialLine = 0,
}

impl StrokeType {
    fn new(num: f64) -> Self {
        let num_base = (num / 100.0).floor() as u32;
        let num_opt = num as u32 % 100;

        if num_opt != 0 {
            return Self::StraightLine;
        };

        match num_base {
            1 => Self::StraightLine,
            2 | 12 => Self::Curve, // 12??
            3 => Self::Polyline,
            4 => Self::OtsuLine,
            6 => Self::CompoundCurve,
            7 => Self::VerticalSweep,
            n => Self::Unknown(n),
        }
    }
}

#[repr(u32)]
enum EndShape {
    /// 開放
    Open = 0,
    /// 連接（橫向）
    HorizontalConnection = 2,
    /// 連接（縱向）
    VerticalConnection = 32,
    /// 左上角
    TopLeftCorner = 12,
    /// 右上角
    TopRightCorner = 22,
    /// 左下角
    BottomLeftCorner = 13,
    /// 右下角
    BottomRightCorner = 23,
    /// 左上挑
    LeftUpwardFlick = 4,
    /// 右上挑
    RightUpwardFlick = 5,
    /// 左下zh用舊
    BottomLeftZhOld = 313,
    /// 左下zh用新
    BottomLeftZhNew = 413,
    /// 右下H/T
    BottomRightHorT = 24,
    /// 細端
    Narrow = 7,
    /// 有屋頂的細入
    RoofedNarrowEntry = 27,
    /// 收筆
    Stop = 8,

    Unknown(u32),
}

impl EndShape {
    fn new(num: f64) -> Self {
        let num_base = (num / 100.0).floor() as u32;
        // let num_opt = num as u32 % 100;

        match num_base {
            0 => Self::Open,
            2 => Self::HorizontalConnection,
            4 => Self::LeftUpwardFlick,
            5 => Self::RightUpwardFlick,
            7 => Self::Narrow,
            8 => Self::Stop,
            12 => Self::TopLeftCorner,
            13 => Self::BottomLeftCorner,
            22 => Self::TopRightCorner,
            23 => Self::BottomRightCorner,
            24 => Self::BottomRightHorT,
            27 => Self::RoofedNarrowEntry,
            32 => Self::VerticalConnection,
            313 => Self::BottomLeftZhOld,
            413 => Self::BottomLeftZhNew,
            n => Self::Unknown(n),
        }
    }
}

// https://glyphwiki.org/wiki/GlyphWiki:KAGE%E3%83%87%E3%83%BC%E3%82%BF%E4%BB%95%E6%A7%98#i3
struct Stroke {
    stroke_type: StrokeType,
    head_shape: EndShape,
    tail_shape: EndShape,
    point1: Point,
    point2: Point,
    point3: Point,
    point4: Point,
}

impl Stroke {
    fn new(
        field1: f64,
        field2: f64,
        field3: f64,
        field4: f64,
        field5: f64,
        field6: f64,
        field7: f64,
        field8: f64,
        field9: f64,
        field10: f64,
        field11: f64,
    ) -> Self {
        let stroke_type = StrokeType::new(field1);
        let head_shape = EndShape::new(field2);
        let tail_shape = EndShape::new(field3);
        let point1 = (field4, field5).into();
        let point2 = (field6, field7).into();
        let point3 = (field8, field9).into();
        let point4 = (field10, field11).into();

        Self {
            stroke_type,
            head_shape,
            tail_shape,
            point1,
            point2,
            point3,
            point4,
        }
    }

    fn get_control_segments(&self) -> Vec<(Point, Point)> {
        match self.stroke_type {
            StrokeType::StraightLine => vec![(self.point1, self.point2)],
            StrokeType::Curve | StrokeType::Polyline | StrokeType::OtsuLine => {
                vec![(self.point1, self.point2), (self.point2, self.point3)]
            }
            StrokeType::CompoundCurve | StrokeType::VerticalSweep => {
                vec![
                    (self.point1, self.point2),
                    (self.point2, self.point3),
                    (self.point3, self.point4),
                ]
            }
            StrokeType::Unknown(_) => vec![],
        }
    }

    fn is_cross<P1, P2>(&self, point_start: P1, point_end: P2) -> bool
    where
        P1: Into<Point> + Copy,
        P2: Into<Point> + Copy,
    {
        self.get_control_segments()
            .iter()
            .any(|(p1, p2)| two_d::is_cross(p1, p2, point_start, point_end))
    }

    fn is_cross_box<P1, P2>(&self, box_diag_1: P1, box_diag_2: P2) -> bool
    where
        P1: Into<Point> + Copy,
        P2: Into<Point> + Copy,
    {
        self.get_control_segments()
            .iter()
            .any(|(p1, p2)| two_d::is_cross_box(p1, p2, box_diag_1, box_diag_2))
    }
}

#[cfg(test)]
mod test {
    use crate::stroke::stretch;

    #[test]
    fn test_stretch() {
        // === min = 0, max = 1000, sp = 200, dp=400；斷點 300->500
        // origin point: 0.0, 150.0, 299.0, 300.0, 650.0, 1000.0
        let result1 = stretch(
            (400.0, 400.0),
            (200.0, 200.0),
            (0.0, 150.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result1, (0.0, 250.0).into());

        let result2 = stretch(
            (400.0, 400.0),
            (200.0, 200.0),
            (299.0, 300.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result2, (498.0, 500.0).into());

        let result3 = stretch(
            (400.0, 400.0),
            (200.0, 200.0),
            (650.0, 1000.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result3, (750.0, 1000.0).into());

        // === dp = sp = 300
        // origin point: 123.0, 400.0, 999.0, 1000.0
        let result4 = stretch(
            (300.0, 300.0),
            (300.0, 300.0),
            (123.0, 400.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result4, (123.0, 400.0).into());

        let result5 = stretch(
            (300.0, 300.0),
            (300.0, 300.0),
            (999.0, 1000.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result5, (999.0, 1000.0).into());

        // === min = -200, max = 800, sp = -100, dp = 300
        // origin point: -200.0, -50.0, 0.0, 800.0
        let result6 = stretch(
            (300.0, 300.0),
            (-100.0, -100.0),
            (-200.0, -50.0),
            (-200.0, -200.0),
            (800.0, 800.0),
        );
        assert_eq!(result6, (-200.0, 250.0).into());

        let result7 = stretch(
            (300.0, 300.0),
            (-100.0, -100.0),
            (0.0, 800.0),
            (-200.0, -200.0),
            (800.0, 800.0),
        );
        assert_eq!(result7, (400.0, 800.0).into());

        // === edge case
        let result8 = stretch(
            (400.0, 400.0),
            (900.0, 900.0),
            (1000.0, 999.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert!(result8.x.is_nan() && result8.y == 499.0);

        // === sp = 500, dp = 100
        // origin point: 300.0, 600.0, 900.0
        let result9 = stretch(
            (100.0, 100.0),
            (500.0, 500.0),
            (300.0, 600.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result9, (100.0, 200.0).into());

        let result10 = stretch(
            (100.0, 100.0),
            (500.0, 500.0),
            (900.0, 900.0),
            (0.0, 0.0),
            (1000.0, 1000.0),
        );
        assert_eq!(result10, (800.0, 800.0).into());
    }
}
