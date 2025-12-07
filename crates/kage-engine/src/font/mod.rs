use crate::{
    font::stroke_adjustment::StrokeAdjustmentTrait,
    line::{
        Line,
        special_line::{SpecialLineType, TransformType},
    },
    polygon::Polygon,
    polygons::Polygons,
    utils::Point,
};

pub mod gothic;
pub mod ming;
pub mod stroke_adjustment;

pub enum Typeface {
    /// https://en.wikipedia.org/wiki/Ming_typefaces
    Ming,
    /// https://en.wikipedia.org/wiki/East_Asian_Gothic_typeface
    Gothic,
}

pub enum FontWrapper {
    MingStyle(ming::Ming),
    GothicStyle(gothic::Gothic),
}

impl FontWrapper {
    pub(crate) fn new(typeface: Typeface, use_curve: bool) -> Self {
        match typeface {
            Typeface::Ming => Self::MingStyle(ming::Ming::new(use_curve)),
            Typeface::Gothic => Self::GothicStyle(gothic::Gothic::new()),
        }
    }

    pub(crate) fn set_use_curve(&mut self, use_curve: bool) {
        match self {
            FontWrapper::MingStyle(ming) => ming.use_curve = use_curve,
            FontWrapper::GothicStyle(_) => {}
        }
    }

    fn select_polygons_rect<P1, P2>(
        polygons: &mut Polygons,
        box_diag_1: P1,
        box_diag_2: P2,
    ) -> Vec<&mut Polygon>
    where
        P1: Into<Point>,
        P2: Into<Point>,
    {
        let box_diag_1 = box_diag_1.into();
        let box_diag_2 = box_diag_2.into();

        polygons
            .array_mut()
            .filter(|polygon| {
                polygon.points().iter().all(|point| {
                    box_diag_1.x <= point.x
                        && point.x <= box_diag_2.x
                        && box_diag_1.y <= point.y
                        && point.y <= box_diag_2.y
                })
            })
            .collect()
    }

    pub(crate) fn df_transform(&self, polygons: &mut Polygons, line_type: SpecialLineType) {
        let polygon_vec =
            Self::select_polygons_rect(polygons, line_type.box_diag_1, line_type.box_diag_2);

        match line_type.transform_type {
            TransformType::HorizontalFlip => {
                let [dx, dy] = [line_type.box_diag_1.x + line_type.box_diag_2.x, 0.0];
                for polygon in polygon_vec {
                    polygon.reflect_x().translate(dx, dy).floor();
                }
            }
            TransformType::VerticalFlip => {
                let [dx, dy] = [0.0, line_type.box_diag_1.y + line_type.box_diag_2.y];
                for polygon in polygon_vec {
                    polygon.reflect_y().translate(dx, dy).floor();
                }
            }
            TransformType::Rotate90 => {
                let [dx, dy] = [
                    line_type.box_diag_1.x + line_type.box_diag_2.y,
                    line_type.box_diag_1.y - line_type.box_diag_1.x,
                ];
                for polygon in polygon_vec {
                    polygon.rotate_90().translate(dx, dy).floor();
                }
            }
            TransformType::Rotate180 => {
                let [dx, dy] = [
                    line_type.box_diag_1.x + line_type.box_diag_2.x,
                    line_type.box_diag_1.y + line_type.box_diag_2.y,
                ];
                for polygon in polygon_vec {
                    polygon.rotate_180().translate(dx, dy).floor();
                }
            }
            TransformType::Rotate270 => {
                let [dx, dy] = [
                    line_type.box_diag_1.x - line_type.box_diag_1.y,
                    line_type.box_diag_2.y + line_type.box_diag_1.x,
                ];
                for polygon in polygon_vec {
                    polygon.rotate_270().translate(dx, dy).floor();
                }
            }
        }
    }

    pub(crate) fn get_drawers<'a>(
        &'a self,
        lines: &'a Vec<Line<'a>>,
    ) -> Vec<Box<dyn Fn(&mut Polygons) + 'a>> {
        match self {
            FontWrapper::MingStyle(ming) => {
                let stroke_refs: Vec<_> = lines
                    .iter()
                    .filter_map(|each| match each {
                        Line::StrokeLine(line_type) => Some(line_type),
                        _ => None,
                    })
                    .collect();
                let stroke_adjustment_vec = ming.adjust_strokes(&stroke_refs);
                let mut stroke_adjustment_iter = stroke_adjustment_vec
                    .iter()
                    .map(|(_, adjusted_stroke)| *adjusted_stroke);

                let mut drawers: Vec<Box<dyn Fn(&mut Polygons) + 'a>> =
                    Vec::with_capacity(lines.len());

                for line in lines {
                    match line {
                        Line::SpecialLine(special_line_type) => {
                            let special_line_type = *special_line_type;
                            drawers.push(Box::new(move |polygons: &mut Polygons| {
                                self.df_transform(polygons, special_line_type)
                            }));
                        }
                        Line::StrokeLine(stroke_line_type) => {
                            let adjusted_stroke = stroke_adjustment_iter
                                .next()
                                .expect("stroke adjustment missing");
                            let stroke_line_type = *stroke_line_type;
                            drawers.push(Box::new(move |polygons: &mut Polygons| {
                                ming.df_draw_font(polygons, stroke_line_type, adjusted_stroke)
                            }));
                        }
                        Line::ComponentReferenceLine(_) | Line::Unknown => {}
                    }
                }

                drawers
            }
            FontWrapper::GothicStyle(gothic) => {
                let mut drawers: Vec<Box<dyn Fn(&mut Polygons) + 'a>> =
                    Vec::with_capacity(lines.len());

                for line in lines {
                    match line {
                        Line::SpecialLine(special_line_type) => {
                            let special_line_type = *special_line_type;
                            drawers.push(Box::new(move |polygons: &mut Polygons| {
                                self.df_transform(polygons, special_line_type)
                            }));
                        }
                        Line::StrokeLine(stroke_line_type) => {
                            drawers.push(Box::new(move |polygons: &mut Polygons| {
                                gothic.df_draw_font(polygons, *stroke_line_type)
                            }));
                        }
                        Line::ComponentReferenceLine(_) | Line::Unknown => {}
                    }
                }

                drawers
            }
        }
    }
}
