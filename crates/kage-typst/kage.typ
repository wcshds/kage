#let p = plugin("./kage_typst.wasm")
#let kage-to-svg(data, name, typeface: "Ming", use-curve: false) = {
  let joined-data = data.pairs().map(each => each.at(0) + "|" + each.at(1).replace(regex(`\n+`.text), "$")).join("\n")
  let use-curve = if use-curve {
    "true"
  } else {
    "false"
  }

  p.kage_to_svg(bytes(joined-data), bytes(name), bytes(typeface), bytes(use-curve))
}

#let data = yaml("./glyph-data.yaml").pairs().map(each => (each.at(0), each.at(1).replace(" ", "$"))).to-dict()

#let kg(name) = context box(
  image(kage-to-svg(data, name.text, typeface: "Ming", use-curve: false), width: text.size),
  baseline: text.size / 5.0,
  // fill: luma(180),
);

#set text(
  size: 30pt,
  bottom-edge: "bounds",
)

這是個測試：#kg[𧊌]#kg[𲄒]#kg[𮖒]#kg[𠲇]#kg[𭿦]#kg[𡗿]#kg[𫡡]#kg[𩝦]#kg[𢨘]#kg[𱶒]#kg[𮋸]#kg[𡺚]#kg[𧏮]#kg[𨝰]#kg[𮏼]#kg[𪼮]#kg[𳎜]#kg[𧜁]#kg[𨩛]#kg[𨔣]#kg[𱮥]#kg[𧥳]#kg[𣊸]#kg[𮓺]#kg[𰨜]#kg[𤩕]#kg[𡜪]#kg[𲃋]#kg[𫙩]#kg[𩚷]#kg[𤝑]#kg[𫟤]#kg[𨠿]#kg[𣄹]#kg[𪽾]#kg[𨟶]#kg[𢑆]#kg[𨁬]#kg[𥯵]#kg[𫜸]#kg[𩩑]#kg[𧜄]
