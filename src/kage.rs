use crate::{
    component::Components,
    font::{ming::Ming, stroke_adjustment::StrokeAdjustmentTrait},
    line::{
        Line,
        stroke_line::{self, StrokeLineType},
    },
    polygons::Polygons,
    utils::{Point, Vector},
};
use core::f64;

pub struct Kage {
    pub components: Components,
    pub font: Ming,
}

impl Kage {
    pub fn new(use_curve: bool) -> Self {
        let font = Ming {
            k_rate: 100,
            min_width_horizontal: 2.0,
            min_width_triangle: 2.0,
            min_width_vertical: 6.0,
            width: 5.0,
            square_terminal: 3.0,
            right_sweep_end_scale_factor: 1.1,
            curve_size: 10.0,
            use_curve,
            adjust_foot_left: vec![14.0, 9.0, 5.0, 2.0, 0.0],
            adjust_foot_right: vec![8.0, 6.0, 4.0, 2.0],
            k_adjust_kakato_range_x: 20.0,
            k_adjust_kakato_range_y: vec![1.0, 19.0, 24.0, 30.0],
            k_adjust_kakato_step: 3.0,
            k_adjust_uroko_x: vec![24.0, 20.0, 16.0, 12.0],
            k_adjust_uroko_y: vec![12.0, 11.0, 9.0, 8.0],
            k_adjust_uroko_length: vec![22.0, 36.0, 50.0],
            k_adjust_uroko_length_step: 3.0,
            k_adjust_uroko_line: vec![22.0, 26.0, 30.0],
            k_adjust_uroko2_step: 3.0,
            k_adjust_uroko2_length: 40.0,
            k_adjust_tate_step: 4.0,
            k_adjust_mage_step: 5.0,
        };

        Self {
            components: Components::new(),
            font,
        }
    }

    pub fn set_use_curve(&mut self, use_curve: bool) {
        self.font.use_curve = use_curve;
    }

    pub fn make_glyph_with_component_name(&self, polygons: &mut Polygons, component_name: &str) {
        let glyph_data = match self.components.search(component_name) {
            Some(content) => content,
            None => return,
        };

        self.make_glyph_with_data(polygons, glyph_data);
    }

    pub fn make_glyph_with_data(&self, polygons: &mut Polygons, data: &str) {
        if data.is_empty() {
            return;
        }

        let lines = self.get_each_expanded_line(data);

        let stroke_line_type_arr: Vec<_> = lines
            .iter()
            .filter_map(|each| match each {
                Line::StrokeLine(line_type) => Some(*line_type),
                _ => None,
            })
            .collect();

        let stroke_refs: Vec<_> = stroke_line_type_arr.iter().collect();
        let stroke_adjustment_vec = self.font.adjust_strokes(&stroke_refs);
        let mut stroke_adjustment_iter = stroke_adjustment_vec.iter().map(|each| each.1);

        for line in lines {
            match line {
                Line::SpecialLine(special_line_type) => {
                    self.font.df_transform(polygons, special_line_type)
                }
                Line::StrokeLine(stroke_line_type) => self.font.df_draw_font(
                    polygons,
                    stroke_line_type,
                    stroke_adjustment_iter.next().unwrap(),
                ),
                _ => continue,
            }
        }
    }

    fn get_each_expanded_line<'a>(&'a self, glyph_data: &'a str) -> Vec<Line<'a>> {
        let mut lines = Vec::new();

        for line_data in glyph_data.split('$') {
            match Line::new(line_data) {
                Line::StrokeLine(stroke_line) => lines.push(Line::StrokeLine(stroke_line)),
                Line::SpecialLine(special_line) => lines.push(Line::SpecialLine(special_line)),
                Line::ComponentReferenceLine(component_reference_line) => {
                    if let Some(component_data) = self
                        .components
                        .search(component_reference_line.component_name)
                    {
                        let mut expanded = self.expand_component_strokes(
                            component_data,
                            component_reference_line.box_diag_1,
                            component_reference_line.box_diag_2,
                            component_reference_line.primary_control_point.x,
                            component_reference_line.primary_control_point.y,
                            component_reference_line.secondary_control_point.x,
                            component_reference_line.secondary_control_point.y,
                        );
                        lines.append(&mut expanded);
                    }
                }
                Line::Unknown => {}
            }
        }

        lines
    }

    fn expand_component_strokes<'a>(
        &'a self,
        component_data: &'a str,
        box_diag_1: Point,
        box_diag_2: Point,
        mut sx: f64,
        sy: f64,
        mut sx2: f64,
        mut sy2: f64,
    ) -> Vec<Line<'a>> {
        let mut stroke_lines: Vec<StrokeLineType> = self
            .get_each_expanded_line(component_data)
            .into_iter()
            .filter_map(|line| match line {
                Line::StrokeLine(stroke_line) => Some(stroke_line),
                _ => None,
            })
            .collect();

        if sx != 0.0 || sy != 0.0 {
            if sx > 100.0 {
                sx -= 200.0;
            } else {
                sx2 = 0.0;
                sy2 = 0.0;
            }
        }

        let do_stretch = sx != 0.0 || sy != 0.0;
        if do_stretch && !stroke_lines.is_empty() {
            let stroke_line::Bounds {
                min_point,
                max_point,
            } = Self::get_box(&stroke_lines);
            let dest_pivot = Point::new(sx, sy, None);
            let src_pivot = Point::new(sx2, sy2, None);

            for stroke in &mut stroke_lines {
                stroke.stretch(dest_pivot, src_pivot, min_point, max_point);
            }
        }

        let scale_vector: Vector = ((box_diag_2 - box_diag_1) / 200.0).into();

        for stroke in &mut stroke_lines {
            stroke.point_1 = box_diag_1 + stroke.point_1 * scale_vector;
            stroke.point_2 = box_diag_1 + stroke.point_2 * scale_vector;
            stroke.point_3 = box_diag_1 + stroke.point_3 * scale_vector;
            stroke.point_4 = box_diag_1 + stroke.point_4 * scale_vector;
        }

        stroke_lines.into_iter().map(Line::StrokeLine).collect()
    }

    fn get_box(strokes: &[StrokeLineType]) -> stroke_line::Bounds {
        let mut min_x: f64 = 200.0;
        let mut min_y: f64 = 200.0;
        let mut max_x: f64 = 0.0;
        let mut max_y: f64 = 0.0;

        for stroke in strokes {
            let stroke_line::Bounds {
                min_point,
                max_point,
            } = stroke.get_box();
            min_x = min_x.min(min_point.x);
            max_x = max_x.max(max_point.x);
            min_y = min_y.min(min_point.y);
            max_y = max_y.max(max_point.y);
        }

        stroke_line::Bounds {
            min_point: (min_x, min_y).into(),
            max_point: (max_x, max_y).into(),
        }
    }
}
