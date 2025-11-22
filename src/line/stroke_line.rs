use crate::{two_d, utils::Point};

pub(crate) fn stretch_numeric(
    dest_pivot: f64,
    src_pivot: f64,
    origin_point: f64,
    min: f64,
    max: f64,
) -> f64 {
    let (p1, p2, p3, p4) = if origin_point < src_pivot + 100.0 {
        (min, src_pivot + 100.0, min, dest_pivot + 100.0)
    } else {
        (src_pivot + 100.0, max, dest_pivot + 100.0, max)
    };

    ((origin_point - p1) / (p2 - p1) * (p4 - p3) + p3).floor()
}

pub(crate) fn stretch<P1, P2, P3, P4, P5>(
    dest_pivot: P1,
    src_pivot: P2,
    origin_point: P3,
    min_point: P4,
    max_point: P5,
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
    let min: Point = min_point.into();
    let max: Point = max_point.into();

    let x = stretch_numeric(dest_pivot.x, src_pivot.x, origin_point.x, min.x, max.x);
    let y = stretch_numeric(dest_pivot.y, src_pivot.y, origin_point.y, min.y, max.y);

    Point::new(x, y, origin_point.off_curve)
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum StrokeKind {
    // 1 ~ 6: Stroke Lines
    /// 直線
    StraightLine = 1, // 2 control point
    /// 曲線
    Curve = 2, // 3 control points
    /// 折線
    BendLine = 3, // 3 control points
    /// 折彎（「乙」狀線）
    OtsuCurve = 4, // 3 control points
    /// 二次曲線
    ComplexCurve = 6, // 4 control points
    /// 豎撇
    VerticalSlash = 7, // 4 control points

    /// 未知的筆劃類型
    Unknown,
    // // 99: component reference line
    // /// 部件引用行
    // ComponentReferenceLine = 99,

    // // 0: Special Lines
    // /// 特殊行
    // SpecialLine = 0,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct StrokeType {
    pub(crate) kind: StrokeKind,
    pub(crate) base: u32,
    pub(crate) opt: u32,
}

impl StrokeType {
    pub(crate) fn new(num: f64) -> Self {
        let num_base = num as u32 % 100;
        let num_opt = (num / 100.0).floor() as u32;

        let kind = match num_base {
            1 => StrokeKind::StraightLine,
            2 | 12 => StrokeKind::Curve, // 12????
            3 => StrokeKind::BendLine,
            4 => StrokeKind::OtsuCurve,
            6 => StrokeKind::ComplexCurve,
            7 => StrokeKind::VerticalSlash,
            _ => StrokeKind::Unknown,
        };

        Self {
            base: num_base,
            opt: num_opt,
            kind,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum EndKind {
    /// 開放
    Free = 0,
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

    Temp14 = 14,
    Temp15 = 15,
    Temp1 = 1,
    Temp9 = 9,
    Temp6 = 6,
    Temp17 = 17,

    Unknown = 1000,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct EndType {
    pub(crate) kind: EndKind,
    pub(crate) base: u32,
    pub(crate) opt: u32,
    pub(crate) opt_1: u32,
    pub(crate) opt_2: u32,
    pub(crate) opt_3: u32,
}

impl EndType {
    pub(crate) fn new(num: f64) -> Self {
        let num_base = num as u32 % 100;
        let num_opt = (num / 100.0).floor() as u32;
        let num_opt_1 = num_opt % 10;
        let num_opt_2 = (num_opt / 10) % 10;
        let num_opt_3 = num_opt / 100;

        let kind = match num_base {
            0 => EndKind::Free,
            2 => EndKind::HorizontalConnection,
            4 => EndKind::LeftUpwardFlick,
            5 => EndKind::RightUpwardFlick,
            7 => EndKind::Narrow,
            8 => EndKind::Stop,
            12 => EndKind::TopLeftCorner,
            13 => {
                if num_opt_1 == 4 {
                    // 413
                    EndKind::BottomLeftZhNew
                } else if num_opt_1 == 3 {
                    // 313
                    EndKind::BottomLeftZhOld
                } else {
                    // 13
                    EndKind::BottomLeftCorner
                }
            }
            22 => EndKind::TopRightCorner,
            23 => EndKind::BottomRightCorner,
            24 => EndKind::BottomRightHorT,
            27 => EndKind::RoofedNarrowEntry,
            32 => EndKind::VerticalConnection,
            14 => EndKind::Temp14,
            15 => EndKind::Temp15,
            1 => EndKind::Temp1,
            9 => EndKind::Temp9,
            6 => EndKind::Temp6,
            17 => EndKind::Temp17,
            _ => EndKind::Unknown,
        };

        EndType {
            base: num_base,
            opt: num_opt,
            opt_1: num_opt_1,
            opt_2: num_opt_2,
            opt_3: num_opt_3,
            kind,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Bounds {
    pub(crate) min_point: Point,
    pub(crate) max_point: Point,
}

// https://glyphwiki.org/wiki/GlyphWiki:KAGE%E3%83%87%E3%83%BC%E3%82%BF%E4%BB%95%E6%A7%98#i3
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StrokeLineType {
    pub(crate) stroke_type: StrokeType,
    pub(crate) head_shape: EndType,
    pub(crate) tail_shape: EndType,
    pub(crate) point_1: Point,
    pub(crate) point_2: Point,
    pub(crate) point_3: Point,
    pub(crate) point_4: Point,
}

impl StrokeLineType {
    pub(crate) fn new(
        field_1: f64,
        field_2: f64,
        field_3: f64,
        field_4: f64,
        field_5: f64,
        field_6: f64,
        field_7: f64,
        field_8: f64,
        field_9: f64,
        field_10: f64,
        field_11: f64,
    ) -> Self {
        let stroke_type = StrokeType::new(field_1);
        let head_shape = EndType::new(field_2);
        let tail_shape = EndType::new(field_3);
        let point_1 = (field_4, field_5, None).into();
        let point_2 = (field_6, field_7, None).into();
        let point_3 = (field_8, field_9, None).into();
        let point_4 = (field_10, field_11, None).into();

        Self {
            stroke_type,
            head_shape,
            tail_shape,
            point_1,
            point_2,
            point_3,
            point_4,
        }
    }

    fn get_control_segments(&self) -> Vec<(Point, Point)> {
        let opt = self.stroke_type.opt;

        if opt != 0 {
            return vec![(self.point_1, self.point_2)];
        }

        match self.stroke_type.kind {
            StrokeKind::StraightLine => vec![(self.point_1, self.point_2)],
            StrokeKind::Curve | StrokeKind::BendLine | StrokeKind::OtsuCurve => {
                vec![(self.point_1, self.point_2), (self.point_2, self.point_3)]
            }
            StrokeKind::ComplexCurve | StrokeKind::VerticalSlash => {
                vec![
                    (self.point_1, self.point_2),
                    (self.point_2, self.point_3),
                    (self.point_3, self.point_4),
                ]
            }
            StrokeKind::Unknown => vec![],
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

    pub(crate) fn stretch<P1, P2, P3, P4>(
        &mut self,
        dest_pivot: P1,
        src_pivot: P2,
        min_point: P3,
        max_point: P4,
    ) where
        P1: Into<Point> + Copy,
        P2: Into<Point> + Copy,
        P3: Into<Point> + Copy,
        P4: Into<Point> + Copy,
    {
        self.point_1 = stretch(dest_pivot, src_pivot, self.point_1, min_point, max_point);
        self.point_2 = stretch(dest_pivot, src_pivot, self.point_2, min_point, max_point);
        // if !(this.a1_100 === 99 && this.a1_opt === 0) {  }
        self.point_3 = stretch(dest_pivot, src_pivot, self.point_3, min_point, max_point);
        self.point_4 = stretch(dest_pivot, src_pivot, self.point_4, min_point, max_point);
    }

    pub(crate) fn get_box(&self) -> Bounds {
        let mut min_point = Point::INFINITY;
        let mut max_point = Point::NEG_INFINITY;

        #[inline]
        fn update_bounds(min_point: &mut Point, max_point: &mut Point, point: Point) {
            *min_point = min_point.min(point);
            *max_point = max_point.max(point);
        }

        // let a1 = if self.a1_opt == 0 { self.a1_100 } else { 6 };
        if self.stroke_type.opt != 0 {
            update_bounds(&mut min_point, &mut max_point, self.point_1);
            update_bounds(&mut min_point, &mut max_point, self.point_2);
            update_bounds(&mut min_point, &mut max_point, self.point_3);
            update_bounds(&mut min_point, &mut max_point, self.point_4);

            return Bounds {
                min_point,
                max_point,
            };
        }

        match self.stroke_type.kind {
            StrokeKind::Unknown if self.stroke_type.base == 0 => {}
            StrokeKind::StraightLine => {
                update_bounds(&mut min_point, &mut max_point, self.point_1);
                update_bounds(&mut min_point, &mut max_point, self.point_2);
            }
            StrokeKind::Curve | StrokeKind::BendLine | StrokeKind::OtsuCurve => {
                update_bounds(&mut min_point, &mut max_point, self.point_1);
                update_bounds(&mut min_point, &mut max_point, self.point_2);
                update_bounds(&mut min_point, &mut max_point, self.point_3);
            }
            _ => {
                update_bounds(&mut min_point, &mut max_point, self.point_1);
                update_bounds(&mut min_point, &mut max_point, self.point_2);
                update_bounds(&mut min_point, &mut max_point, self.point_3);
                update_bounds(&mut min_point, &mut max_point, self.point_4);
            }
        }

        Bounds {
            min_point,
            max_point,
        }
    }
}

#[cfg(test)]
mod test {
    use core::f64;

    use crate::{
        line::stroke_line::{
            Bounds, EndKind, EndType, StrokeKind, StrokeLineType, StrokeType, stretch,
        },
        utils::Point,
    };

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

    #[test]
    fn test_construction() {
        let stroke1 =
            StrokeLineType::new(1.0, 0.0, 2.0, 32.0, 31.0, 176.0, 31.0, 0.0, 0.0, 0.0, 0.0);

        assert_eq!(
            stroke1,
            StrokeLineType {
                stroke_type: StrokeType {
                    base: 1,
                    opt: 0,
                    kind: StrokeKind::StraightLine,
                },
                head_shape: EndType {
                    base: 0,
                    opt: 0,
                    opt_1: 0,
                    opt_2: 0,
                    opt_3: 0,
                    kind: EndKind::Free,
                },
                tail_shape: EndType {
                    base: 2,
                    opt: 0,
                    opt_1: 0,
                    opt_2: 0,
                    opt_3: 0,
                    kind: EndKind::HorizontalConnection,
                },
                point_1: Point {
                    x: 32.0,
                    y: 31.0,
                    off_curve: None,
                },
                point_2: Point {
                    x: 176.0,
                    y: 31.0,
                    off_curve: None,
                },
                point_3: Point {
                    x: 0.0,
                    y: 0.0,
                    off_curve: None,
                },
                point_4: Point {
                    x: 0.0,
                    y: 0.0,
                    off_curve: None,
                },
            }
        );

        assert_eq!(
            stroke1.get_control_segments(),
            vec![(
                Point {
                    x: 32.0,
                    y: 31.0,
                    off_curve: None,
                },
                Point {
                    x: 176.0,
                    y: 31.0,
                    off_curve: None,
                },
            )]
        );

        assert_eq!(
            stroke1.get_box(),
            Bounds {
                min_point: Point {
                    x: 32.0,
                    y: 31.0,
                    off_curve: None,
                },
                max_point: Point {
                    x: 176.0,
                    y: 31.0,
                    off_curve: None,
                },
            }
        )
    }

    #[test]
    fn test_different_stroke_type() {
        let stroke2 = StrokeLineType::new(
            2.0, 22.0, 7.0, 176.0, 31.0, 170.0, 43.0, 156.0, 63.0, 0.0, 0.0,
        );

        assert_eq!(stroke2.stroke_type.kind, StrokeKind::Curve);
        assert_eq!(stroke2.stroke_type.base, 2);
        assert_eq!(stroke2.stroke_type.opt, 0);
        assert_eq!(
            stroke2.get_control_segments(),
            [
                (
                    Point {
                        x: 176.0,
                        y: 31.0,
                        off_curve: None,
                    },
                    Point {
                        x: 170.0,
                        y: 43.0,
                        off_curve: None,
                    },
                ),
                (
                    Point {
                        x: 170.0,
                        y: 43.0,
                        off_curve: None,
                    },
                    Point {
                        x: 156.0,
                        y: 63.0,
                        off_curve: None,
                    },
                ),
            ]
        );

        let stroke3 = StrokeLineType::new(
            3.0, 0.0, 0.0, 100.0, 100.0, 150.0, 50.0, 200.0, 100.0, 250.0, 150.0,
        );

        assert_eq!(stroke3.stroke_type.kind, StrokeKind::BendLine);
        assert_eq!(stroke3.stroke_type.base, 3);
        assert_eq!(stroke3.stroke_type.opt, 0);
        assert_eq!(
            stroke3.get_control_segments(),
            vec![
                (
                    Point {
                        x: 100.0,
                        y: 100.0,
                        off_curve: None,
                    },
                    Point {
                        x: 150.0,
                        y: 50.0,
                        off_curve: None,
                    },
                ),
                (
                    Point {
                        x: 150.0,
                        y: 50.0,
                        off_curve: None,
                    },
                    Point {
                        x: 200.0,
                        y: 100.0,
                        off_curve: None,
                    },
                ),
            ]
        );
        assert_eq!(
            stroke3.get_box(),
            Bounds {
                min_point: Point {
                    x: 100.0,
                    y: 50.0,
                    off_curve: None,
                },
                max_point: Point {
                    x: 200.0,
                    y: 100.0,
                    off_curve: None,
                },
            }
        )
    }

    #[test]
    fn test_cross() {
        let stroke4 =
            StrokeLineType::new(1.0, 0.0, 0.0, 0.0, 0.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0);

        assert_eq!(stroke4.is_cross((50.0, 50.0), (150.0, 150.0)), false);
        assert_eq!(stroke4.is_cross((200.0, 200.0), (300.0, 300.0)), false);
        assert_eq!(stroke4.is_cross_box((25.0, 25.0), (75.0, 75.0)), true);
        assert_eq!(stroke4.is_cross_box((200.0, 200.0), (300.0, 300.0)), false);
    }

    #[test]
    fn test_stroke_stretch() {
        let mut stroke6 =
            StrokeLineType::new(1.0, 0.0, 0.0, 50.0, 50.0, 100.0, 100.0, 0.0, 0.0, 0.0, 0.0);

        assert_eq!(
            stroke6.get_box(),
            Bounds {
                min_point: Point {
                    x: 50.0,
                    y: 50.0,
                    off_curve: None,
                },
                max_point: Point {
                    x: 100.0,
                    y: 100.0,
                    off_curve: None,
                },
            }
        );

        stroke6.stretch((0.0, 0.0), (0.0, 0.0), (0.0, 0.0), (200.0, 200.0));

        assert_eq!(
            stroke6.get_box(),
            Bounds {
                min_point: Point {
                    x: 50.0,
                    y: 50.0,
                    off_curve: None,
                },
                max_point: Point {
                    x: 100.0,
                    y: 100.0,
                    off_curve: None,
                },
            }
        );
    }

    #[test]
    fn test_edge_case() {
        let stroke7 = StrokeLineType::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

        assert_eq!(stroke7.get_control_segments(), vec![]);
        assert_eq!(
            stroke7.get_box(),
            Bounds {
                min_point: Point {
                    x: f64::INFINITY,
                    y: f64::INFINITY,
                    off_curve: None,
                },
                max_point: Point {
                    x: f64::NEG_INFINITY,
                    y: f64::NEG_INFINITY,
                    off_curve: None,
                },
            }
        );
    }

    #[test]
    fn test_large_number() {
        let stroke8 = StrokeLineType::new(
            1.0, 0.0, 0.0, 1000.0, 1000.0, 2000.0, 2000.0, 0.0, 0.0, 0.0, 0.0,
        );

        assert_eq!(
            stroke8.get_box(),
            Bounds {
                min_point: Point {
                    x: 1000.0,
                    y: 1000.0,
                    off_curve: None,
                },
                max_point: Point {
                    x: 2000.0,
                    y: 2000.0,
                    off_curve: None,
                },
            }
        );
    }

    #[test]
    fn test_negtive_number() {
        let stroke9 = StrokeLineType::new(
            1.0, 0.0, 0.0, -100.0, -100.0, -50.0, -50.0, 0.0, 0.0, 0.0, 0.0,
        );

        assert_eq!(
            stroke9.get_box(),
            Bounds {
                min_point: Point {
                    x: -100.0,
                    y: -100.0,
                    off_curve: None,
                },
                max_point: Point {
                    x: -50.0,
                    y: -50.0,
                    off_curve: None,
                },
            }
        );
    }
}
