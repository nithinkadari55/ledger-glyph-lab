// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#let section(title, items) = block(width: 100%)[
  #block(width: 100%, fill: luma(235), inset: (x: 7pt, y: 5pt), below: 0.6em)[*#upper(title)*]
  #stack(spacing: 0.7em, ..items.map(i => [☐ #i]))
]
#stack(spacing: 1.3em,
  section("Clothing", ("Hiking shirts", "Trousers", "Underwear")),
  section("Footwear", ("Boots", "Socks", "Sandals")),
)
