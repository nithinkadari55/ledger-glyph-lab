// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#let round(title, body) = block(width: 100%, height: 150pt, stroke: 0.7pt + gray, radius: 3pt, clip: true)[
  #block(width: 100%, height: 34pt, fill: rgb("#e8eef5"), inset: 9pt, below: 0pt)[*#title*]
  #block(inset: 9pt, body)
]
#grid(
  columns: (1fr, 1fr),
  column-gutter: 12pt,
  round[Round 1][Which river runs through Vienna? \ Which planet has the most moons?],
  round[Round 2][Who painted the Night Watch? \ Which metal melts in the hand?],
)
#v(12pt)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 12pt,
  [
    #block(width: 100%, height: 34pt, stroke: (bottom: 1.5pt + orange), inset: (y: 7pt))[#text(size: 14pt, weight: "bold")[Results]]
    #lorem(45)
  ],
  block(width: 100%, height: 120pt, fill: rgb("#e8eef5"), inset: 10pt)[*Key finding* \ #lorem(22)],
)
#lorem(120)
