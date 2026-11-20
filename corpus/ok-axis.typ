// expect-none
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#let piece = box(width: 100%, height: 12pt)[
  #place(horizon, line(length: 100%, stroke: 2pt))
  #place(center + horizon, circle(radius: 4pt, fill: orange))
]
#grid(columns: (1fr,) * 6, row-gutter: 0pt,
  ..range(6).map(i => align(center)[*#(1817 + 20 * i)*]),
  ..range(6).map(_ => piece),
  ..range(6).map(_ => align(center, line(angle: 90deg, length: 40pt, stroke: 1pt + orange))),
)
