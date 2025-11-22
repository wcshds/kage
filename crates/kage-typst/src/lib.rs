use kage_engine::{kage::Kage, polygons::Polygons};
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn kage_to_svg(component_data: &[u8], name: &[u8]) -> Vec<u8> {
    let component_data: &str = unsafe { core::str::from_utf8_unchecked(component_data) };
    let name = unsafe { core::str::from_utf8_unchecked(name) };

    let mut kage = Kage::new(false);
    let mut polygons = Polygons::new();

    for line in component_data.trim().split("\n") {
        let mut tmp = line.split("|");
        let name = match tmp.next() {
            Some(content) => content,
            None => continue,
        };
        let glyph_data = match tmp.next() {
            Some(content) => content,
            None => continue,
        };
        kage.components.push(name, glyph_data);
    }

    kage.make_glyph_with_component_name(&mut polygons, name);

    polygons.generate_svg(false).into_bytes()
}
