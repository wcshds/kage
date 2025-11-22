pub(crate) mod component_reference_line;
pub(crate) mod special_line;
pub(crate) mod stroke_line;

use crate::line::{
    component_reference_line::ComponentReferenceLine,
    special_line::{SpecialLineType, TransformType},
    stroke_line::{StrokeKind, StrokeLineType},
};

#[derive(Debug, PartialEq)]
pub enum Line<'a> {
    /// 特殊行
    SpecialLine(SpecialLineType),
    /// 筆劃行
    StrokeLine(StrokeLineType),
    /// 部件引用行
    ComponentReferenceLine(ComponentReferenceLine<'a>),
    Unknown,
}

impl<'a> Line<'a> {
    pub fn new(line_data: &'a str) -> Self {
        #[derive(Clone, Copy)]
        enum FieldType<'a> {
            Num(f64),
            Str(&'a str),
        }
        let mut fields = line_data
            // I think this implementation is very naïve, but this is
            // what the initial JavaScript version does.
            .split(':')
            .map(|each| {
                if let Ok(parsed) = each.parse::<f64>() {
                    FieldType::Num(parsed)
                } else {
                    FieldType::Str(each)
                }
            })
            .chain(core::iter::repeat(FieldType::Num(0.0)))
            .take(11);

        macro_rules! next_num_field {
            ($fields:expr) => {{
                match $fields.next() {
                    Some(FieldType::Num(field)) => field.floor(),
                    _ => return Line::Unknown,
                }
            }};
        }

        macro_rules! get_field_num {
            ($field:expr) => {{
                match $field {
                    FieldType::Num(field) => field.floor(),
                    _ => return Line::Unknown,
                }
            }};
        }

        macro_rules! get_field_str {
            ($field:expr) => {{
                match $field {
                    FieldType::Str(field) => field,
                    _ => return Line::Unknown,
                }
            }};
        }

        let field_1 = next_num_field!(fields);
        let field_2 = next_num_field!(fields);
        let field_3 = next_num_field!(fields);
        let field_4 = next_num_field!(fields);
        let field_5 = next_num_field!(fields);
        let field_6 = next_num_field!(fields);
        let field_7 = next_num_field!(fields);
        let field_8 = fields
            .next()
            .expect("The length of `fields` is equal to 11."); // field 7 is special
        let field_9 = next_num_field!(fields);
        let field_10 = next_num_field!(fields);
        let field_11 = next_num_field!(fields);

        match (field_1 as u32, field_2 as u32, field_3 as u32) {
            (99, _, _) => Line::ComponentReferenceLine(ComponentReferenceLine {
                component_name: get_field_str!(field_8),
                box_diag_1: (field_4, field_5, None).into(),
                box_diag_2: (field_6, field_7, None).into(),
                primary_control_point: (field_2, field_3, None).into(), // None or Some?
                secondary_control_point: (field_10, field_11, None).into(), // None or Some?
            }),
            // special line
            (0, 99, 1) => Line::SpecialLine(SpecialLineType {
                transform_type: TransformType::Rotate90,
                box_diag_1: (field_4, field_5, None).into(),
                box_diag_2: (field_6, field_7, None).into(),
            }),
            (0, 99, 2) => Line::SpecialLine(SpecialLineType {
                transform_type: TransformType::Rotate180,
                box_diag_1: (field_4, field_5, None).into(),
                box_diag_2: (field_6, field_7, None).into(),
            }),
            (0, 99, 3) => Line::SpecialLine(SpecialLineType {
                transform_type: TransformType::Rotate270,
                box_diag_1: (field_4, field_5, None).into(),
                box_diag_2: (field_6, field_7, None).into(),
            }),
            (0, 98, 0) => Line::SpecialLine(SpecialLineType {
                transform_type: TransformType::HorizontalFlip,
                box_diag_1: (field_4, field_5, None).into(),
                box_diag_2: (field_6, field_7, None).into(),
            }),
            (0, 97, 0) => Line::SpecialLine(SpecialLineType {
                transform_type: TransformType::VerticalFlip,
                box_diag_1: (field_4, field_5, None).into(),
                box_diag_2: (field_6, field_7, None).into(),
            }),
            _ => {
                let stroke_result = StrokeLineType::new(
                    field_1,
                    field_2,
                    field_3,
                    field_4,
                    field_5,
                    field_6,
                    field_7,
                    get_field_num!(field_8),
                    field_9,
                    field_10,
                    field_11,
                );

                match stroke_result.stroke_type.kind {
                    StrokeKind::Unknown => Line::Unknown,
                    _ => Line::StrokeLine(stroke_result),
                }
            }
        }
    }
}

pub trait LineVecTrait {
    fn generate_kage(&self) -> String;
}

impl<'a> LineVecTrait for Vec<Line<'a>> {
    fn generate_kage(&self) -> String {
        let mut result = String::new();

        for line in self {
            match line {
                Line::SpecialLine(special_line) => {
                    match special_line.transform_type {
                        TransformType::VerticalFlip => result.push_str("0:97:0:"),
                        TransformType::HorizontalFlip => result.push_str("0:98:0:"),
                        TransformType::Rotate90 => result.push_str("0:99:1:"),
                        TransformType::Rotate180 => result.push_str("0:99:2:"),
                        TransformType::Rotate270 => result.push_str("0:99:3:"),
                    }
                    result.push_str(&format!(
                        "{}:{}:{}:{}",
                        special_line.box_diag_1.x,
                        special_line.box_diag_1.y,
                        special_line.box_diag_2.x,
                        special_line.box_diag_2.y
                    ));
                    result.push('$');
                }
                Line::ComponentReferenceLine(component_reference_line) => {
                    result.push_str("99:");
                    result.push_str(&format!(
                        "{}:{}:",
                        component_reference_line.primary_control_point.x,
                        component_reference_line.primary_control_point.y,
                    ));
                    result.push_str(&format!(
                        "{}:{}:",
                        component_reference_line.box_diag_1.x,
                        component_reference_line.box_diag_1.y,
                    ));
                    result.push_str(&format!(
                        "{}:{}:",
                        component_reference_line.box_diag_2.x,
                        component_reference_line.box_diag_2.y,
                    ));
                    result.push_str(&format!(
                        "0:{}:{}",
                        component_reference_line.secondary_control_point.x,
                        component_reference_line.secondary_control_point.y,
                    ));
                    result.push('$');
                }
                Line::StrokeLine(stroke_line) => {
                    result.push_str(&format!(
                        "{}:{}:{}:",
                        stroke_line.stroke_type.base,
                        stroke_line.head_shape.base + stroke_line.head_shape.opt * 100,
                        stroke_line.tail_shape.base + stroke_line.tail_shape.opt * 100,
                    ));
                    result.push_str(&format!(
                        "{}:{}:{}:{}",
                        stroke_line.point_1.x,
                        stroke_line.point_1.y,
                        stroke_line.point_2.x,
                        stroke_line.point_2.y,
                    ));
                    match stroke_line.stroke_type.kind {
                        StrokeKind::Curve | StrokeKind::BendLine | StrokeKind::OtsuCurve => {
                            result.push_str(&format!(
                                ":{}:{}",
                                stroke_line.point_3.x, stroke_line.point_3.y,
                            ));
                        }
                        StrokeKind::ComplexCurve | StrokeKind::VerticalSlash => {
                            result.push_str(&format!(
                                ":{}:{}:{}:{}",
                                stroke_line.point_3.x,
                                stroke_line.point_3.y,
                                stroke_line.point_4.x,
                                stroke_line.point_4.y,
                            ));
                        }
                        StrokeKind::StraightLine | StrokeKind::Unknown => {}
                    }
                    result.push('$');
                }
                _ => {}
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use crate::line::Line;

    #[test]
    fn test_line_init() {
        let line_special_rotate_270 = Line::new("0:99:3:0:0:200:200:0:0:0:0");
        let line_special_rotate_180 = Line::new("0:99:2:0:0:200:200:0:0:0:0");
        let line_special_rotate_90 = Line::new("0:99:1:0:0:200:200:0:0:0:0");
        let line_component_reference = Line::new("99:0:0:41:0:172:200:u4e3f-07:0:0:0");
        let line_stroke_1 = Line::new("7:0:7:99:17:99:79:99:158:18:188");
        let line_stroke_2 = Line::new("4:22:5:74:87:77:180:184:180");
        let line_stroke_3 = Line::new("1:0:0:26:42:87:42");

        println!("{:#?}", line_special_rotate_270);
        println!("{:#?}", line_special_rotate_180);
        println!("{:#?}", line_special_rotate_90);
        println!("{:#?}", line_component_reference);
        println!("{:#?}", line_stroke_1);
        println!("{:#?}", line_stroke_2);
        println!("{:#?}", line_stroke_3);
    }
}
