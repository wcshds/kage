use crate::utils::Point;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum TransformType {
    Rotate90,
    Rotate180,
    Rotate270,
    HorizontalFlip,
    VerticalFlip,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpecialLineType {
    pub(crate) transform_type: TransformType,
    pub(crate) box_diag_1: Point,
    pub(crate) box_diag_2: Point,
}
