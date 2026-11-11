// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#let card(name, body, price) = block(width: 100%, height: 190pt, fill: luma(238), inset: 10pt)[
  *#name*

  #body

  Weight 1.8 kg \
  Packed size 46 cm

  #v(1fr)
  #align(right, text(size: 14pt, weight: "bold", price))
]
#grid(
  columns: (1fr, 1fr),
  column-gutter: 14pt,
  card[Tarn tent][A freestanding tent for two, pitched in four minutes.][£349],
  card[Ember bag][A down bag with a draft collar for cold shoulders.][£289],
)
#lorem(150)
