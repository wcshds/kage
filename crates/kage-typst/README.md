# kage-typst
A plugin to introduce kage engine to Typst.

## Build
Set up the Rust toolchain, add the WebAssembly target:

```bash
rustup target add wasm32-unknown-unknown
```

Build the WebAssembly module.

```bash
cargo build --release --package kage-typst --target wasm32-unknown-unknown
cp ./target/wasm32-unknown-unknown/release/kage_typst.wasm ./crates/kage-typst/
```

Compile it to see how the KAGE engine is used from Typst.

```bash
typst compile ./crates/kage-typst/kage.typ
```