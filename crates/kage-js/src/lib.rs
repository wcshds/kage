mod utils;

use kage_engine::{kage::Kage, polygons::Polygons};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn kage_to_svg(component_data: JsValue, name: &str, use_curve: bool) -> String {
    let component_data: Vec<(String, String)> =
        serde_wasm_bindgen::from_value(component_data).expect("invalid data");
    let mut kage = Kage::new(use_curve);
    let mut polygons = Polygons::new();

    for (name, glyph_data) in component_data {
        kage.components.push(name, glyph_data);
    }

    kage.make_glyph_with_component_name(&mut polygons, name);

    polygons.generate_svg(use_curve)
}
