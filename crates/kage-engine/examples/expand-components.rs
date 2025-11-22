use std::fs;

use kage_engine::{kage::Kage, line::LineVecTrait};

fn main() {
    let mut kage = Kage::new(false);
    let newest_data = fs::read_to_string("./data/dump_newest_only.txt").unwrap();
    let all_version_data = fs::read_to_string("./data/dump_all_versions.txt").unwrap();
    set_full_components(&mut kage, &newest_data, &all_version_data);

    let glyph_data = match kage.components.search("u30ede") {
        Some(content) => content,
        None => "",
    };
    
    let lines = kage.get_each_expanded_line(glyph_data);
    println!("{}", lines.generate_kage());
}

fn set_full_components<'a>(
    kage: &mut Kage,
    newest_data: &'a str,
    all_version_data: &'a str,
) -> Vec<&'a str> {
    let mut result = Vec::new();

    for line in newest_data.lines().skip(2) {
        let mut splits = line.split("|");
        let name = match splits.next() {
            Some(content) => content.trim(),
            None => continue,
        };
        let _related = match splits.next() {
            Some(content) => content.trim(),
            None => continue,
        };
        let glyph_data = match splits.next() {
            Some(content) => content.trim(),
            None => continue,
        };

        kage.components.push(name, glyph_data);
        result.push(name);
    }

    for line in all_version_data.lines().skip(2) {
        let mut splits = line.split("|");
        let name = match splits.next() {
            Some(content) => content.trim().replace(r#"\@"#, "@"),
            None => continue,
        };
        let _related = match splits.next() {
            Some(content) => content.trim(),
            None => continue,
        };
        let glyph_data = match splits.next() {
            Some(content) => content.trim(),
            None => continue,
        };

        kage.components.push(name, glyph_data);
    }

    result
}
