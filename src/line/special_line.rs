use crate::utils::Point;

#[derive(Debug, PartialEq)]
pub(crate) enum TransformType {
    Rotate90,
    Rotate180,
    Rotate270,
    HorizontalFlip,
    VerticalFlip,
}

#[derive(Debug, PartialEq)]
pub struct SpecialLineType {
    pub(crate) transform_type: TransformType,
    pub(crate) box_diag_1: Point,
    pub(crate) box_diag_2: Point,
}
