use crate::utils::Point;

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
