use crate::utils::Point;

#[derive(Debug, PartialEq)]
pub(crate) struct ComponentReferenceLine<'a> {
    pub(crate) component_name: &'a str,
    pub(crate) box_diag_1: Point,
    pub(crate) box_diag_2: Point,
    pub(crate) primary_control_point: Point,   // point D
    pub(crate) secondary_control_point: Point, // point S
}
