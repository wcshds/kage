use std::fs;

use kage::{kage::Kage, polygons::Polygons};

fn main() {
    let mut kage = Kage::new(false);
    let newest_data = fs::read_to_string("./data/dump_newest_only.txt").unwrap();
    let all_version_data = fs::read_to_string("./data/dump_all_versions.txt").unwrap();
    let names = set_full_components(&mut kage, &newest_data, &all_version_data);

    let mut polygons = Polygons::new();

    for (idx, name) in names.iter().enumerate() {
        let sub_dir_num = idx / 10000;
        if idx % 10000 == 0 {
            fs::create_dir_all(format!("./results/{:03}", sub_dir_num)).unwrap()
        }

        polygons.clear();
        kage.make_glyph_with_component_name(&mut polygons, name);
        let result = polygons.generate_svg(false);
        fs::write(
            format!("./results/{:03}/{:07}_{}.svg", sub_dir_num, idx, name),
            result,
        )
        .unwrap();
    }
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
