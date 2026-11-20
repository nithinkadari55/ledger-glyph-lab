// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#let card(title, n) = block(width: 100%, height: 220pt, fill: white, stroke: 0.6pt + luma(160), radius: 4pt, clip: true)[
  #block(width: 100%, height: 60pt, fill: rgb("#2f5a4a"), inset: 10pt)[#text(fill: white, weight: 700)[#title]]
  #pad(10pt)[#for i in range(n) [Specification line #(i + 1) \ ]]
]
#grid(columns: (1fr, 1fr), column-gutter: 12pt,
  card("Ridgeline 2 Tent", 6), card("Emberdown Bag", 6))
