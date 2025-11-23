#let p = plugin("./kage_typst.wasm")
#let kage-to-svg(data, name, use-curve: false) = {
  let joined-data = data.pairs().map(each => each.at(0) + "|" + each.at(1).replace(regex(`\n+`.text), "$")).join("\n")
  let use-curve = if use-curve {
    "true"
  } else {
    "false"
  }

  p.kage_to_svg(bytes(joined-data), bytes(name), bytes(use-curve))
}

#let data = yaml("./glyph-data.yaml").pairs().map(each => (each.at(0), each.at(1).replace(" ", "$"))).to-dict()

#image(kage-to-svg(data, "u5f41"))
#image(kage-to-svg(data, "u2a1c4", use-curve: true))
