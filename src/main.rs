use kage::{
    font::{ming::Ming, stroke_adjustment::StrokeAdjustmentTrait},
    line::Line,
    polygons::Polygons,
};

fn init(use_curve: bool) -> Ming {
    Ming {
        k_rate: 100,
        k_min_width_horizontal: 2.0,
        k_min_width_triangle: 2.0,
        k_min_width_vertical: 6.0,
        k_width: 5.0,
        k_square_terminal: 3.0,
        k_l2rdfatten: 1.1,
        k_mage: 10.0,
        k_use_curve: use_curve,
        k_adjust_kakato_l: vec![14.0, 9.0, 5.0, 2.0, 0.0],
        k_adjust_kakato_r: vec![8.0, 6.0, 4.0, 2.0],
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
    }
}

fn main() {
    let glyph_data = "6:32:7:128:28:145:28:136:54:131:68$2:7:8:131:67:193:88:165:150$2:7:8:110:173:150:187:168:144$2:7:8:127:176:102:176:106:129$2:32:7:105:133:111:97:120:89$2:7:8:117:90:137:85:129:138$2:32:7:130:133:120:176:97:172$2:7:8:108:174:81:176:80:153$2:32:7:80:158:83:117:142:135$2:7:8:85:141:118:108:178:172$2:7:8:162:188:170:180:177:168$2:7:8:43:42:34:119:67:165$2:7:8:73:26:80:104:41:176$2:7:8:25:160:29:174:43:173$2:7:8:84:51:31:106:23:161$2:7:8:36:176:24:165:23:157$2:7:8:77:60:108:29:132:28";

    let line_arr: Vec<_> = glyph_data
        .split("$")
        .map(|field_data| Line::new(field_data))
        .collect();
    let stroke_line_type_arr: Vec<_> = line_arr
        .iter()
        .filter_map(|each| match each {
            Line::StrokeLine(line_type) => Some(line_type),
            _ => None,
        })
        .collect();

    let ming = init(false);

    let result = ming.adjust_strokes(&stroke_line_type_arr);
    let mut polygons = Polygons::new();
    for (&stroke_line_type, adjusted_stroke) in result {
        ming.df_draw_font(
            &mut polygons,
            Line::StrokeLine(stroke_line_type),
            adjusted_stroke,
        );
    }

    println!("{}", polygons.generate_svg(false));
}
