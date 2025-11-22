# Kage-rs

An experimental port of the Kage engine to Rust. This project is still in a very early stage.

## Example

To run the example, you first need to download the GlyphWiki data:

```bash
curl -O https://glyphwiki.org/dump.tar.gz
mkdir ./data
tar -xzf dump.tar.gz -C ./data
```

Then run:

```bash
cargo run --release -p kage-engine --example export-all
```

This will generate SVG files from all raw GlyphWiki Kage data.
