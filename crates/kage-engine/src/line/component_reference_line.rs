use crate::utils::{Point, Rgb};

#[derive(Debug, PartialEq)]
pub struct ComponentReferenceLine<'a> {
    pub(crate) component_name: &'a str,
    pub(crate) box_diag_1: Point,
    pub(crate) box_diag_2: Point,
    pub(crate) primary_control_point: Point,   // point D
    pub(crate) secondary_control_point: Point, // point S
    pub(crate) color: Option<Rgb>,
}
