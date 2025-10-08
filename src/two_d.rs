use crate::utils::{Point, Vector, round};

fn cross<V1, V2>(vector_1: V1, vector_2: V2) -> f64
where
    V1: Into<Vector>,
    V2: Into<Vector>,
{
    let vector_1 = vector_1.into();
    let vector_2 = vector_2.into();
    vector_1.x * vector_2.y - vector_1.y * vector_2.x
}

fn is_cross<P1, P2, P3, P4>(
    vector_1_start: P1,
    vector_1_end: P2,
    vector_2_start: P3,
    vector_2_end: P4,
) -> bool
where
    P1: Into<Point>,
    P2: Into<Point>,
    P3: Into<Point>,
    P4: Into<Point>,
{
    let vector_1_start = vector_1_start.into();
    let vector_1_end = vector_1_end.into();
    let vector_2_start = vector_2_start.into();
    let vector_2_end = vector_2_end.into();

    // 1s: vector_1_start
    // 1e: vector_1_end
    // 2s: vector_2_start
    // 2e: vector_2_end
    let cross_1s1e_2s2e = cross(vector_1_end - vector_1_start, vector_2_end - vector_2_start);
    if cross_1s1e_2s2e.is_nan() {
        return true;
    }
    if cross_1s1e_2s2e == 0.0 {
        // parallel
        return false; // XXX should check if segments overlap?
    }

    let cross_1s1e_1s2s = cross(
        vector_1_end - vector_1_start,
        vector_2_start - vector_1_start,
    );
    let cross_1s1e_1s2e = cross(vector_1_end - vector_1_start, vector_2_end - vector_1_start);
    let cross_2s2e_2s1s = cross(
        vector_2_end - vector_2_start,
        vector_1_start - vector_2_start,
    );
    let cross_2s2e_2s1e = cross(vector_2_end - vector_2_start, vector_1_end - vector_2_start);

    round(cross_1s1e_1s2s * cross_1s1e_1s2e, 5) <= 0.0
        && round(cross_2s2e_2s1s * cross_2s2e_2s1e, 5) <= 0.0
}

fn is_cross_box<P1, P2, P3, P4>(
    vector_1_start: P1,
    vector_1_end: P2,
    box_diag_1: P3,
    box_diag_2: P4,
) -> bool
where
    P1: Into<Point>,
    P2: Into<Point>,
    P3: Into<Point>,
    P4: Into<Point>,
{
    let vector_1_start = vector_1_start.into();
    let vector_1_end = vector_1_end.into();
    let box_diag_1 = box_diag_1.into();
    let box_diag_2 = box_diag_2.into();

    let top_left = box_diag_1;
    let top_right = (box_diag_2.x, box_diag_1.y);
    let bottom_left = (box_diag_1.x, box_diag_2.y);
    let bottom_right = box_diag_2;

    if is_cross(vector_1_start, vector_1_end, top_left, top_right) {
        true
    } else if is_cross(vector_1_start, vector_1_end, top_right, bottom_right) {
        true
    } else if is_cross(vector_1_start, vector_1_end, bottom_left, bottom_right) {
        true
    } else if is_cross(vector_1_start, vector_1_end, top_left, bottom_left) {
        true
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_cross() {
        // true
        assert!(is_cross((0.0, 0.0), (4.0, 4.0), (0.0, 4.0), (4.0, 0.0),));
        // true
        assert!(is_cross((0.0, 0.0), (2.0, 2.0), (2.0, 2.0), (5.0, 0.0),));
        // false
        assert!(!is_cross((0.0, 0.0), (4.0, 0.0), (0.0, 1.0), (4.0, 1.0),));
        // false
        assert!(!is_cross((0.0, 0.0), (4.0, 0.0), (2.0, 0.0), (6.0, 0.0)));
        // false
        assert!(!is_cross((0.0, 0.0), (1.0, 1.0), (2.0, 2.0), (3.0, 5.0),));
        // false
        assert!(!is_cross((1.0, 1.0), (1.0, 1.0), (0.0, 0.0), (2.0, 2.0),));
        // true
        assert!(is_cross(
            (f64::NAN, 0.0),
            (1.0, 1.0),
            (0.0, 0.0),
            (2.0, 2.0),
        ));
    }

    #[test]
    fn test_is_cross_box() {
        // true
        assert!(is_cross_box((0.0, 0.0), (6.0, 5.0), (1.0, 1.0), (5.0, 4.0),));
        // true
        assert!(is_cross_box((0.0, 2.0), (1.0, 2.0), (1.0, 1.0), (5.0, 4.0),));
        // false
        assert!(!is_cross_box(
            (2.0, 2.0),
            (4.0, 3.0),
            (1.0, 1.0),
            (5.0, 4.0),
        ));
        // true
        assert!(is_cross_box((0.0, 1.0), (6.0, 1.0), (1.0, 1.0), (5.0, 4.0),));
        // false
        assert!(!is_cross_box(
            (-2.0, -1.0),
            (-1.0, -3.0),
            (1.0, 1.0),
            (5.0, 4.0),
        ));
        // true
        assert!(is_cross_box(
            (f64::NAN, 0.0),
            (2.0, 2.0),
            (1.0, 1.0),
            (5.0, 4.0),
        ));
        // true
        assert!(is_cross_box((0.0, 3.0), (6.0, 3.0), (5.0, 4.0), (1.0, 1.0),));
    }
}
