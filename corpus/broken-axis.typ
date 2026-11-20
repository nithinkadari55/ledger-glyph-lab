// expect: L102.broken-axis
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#let piece(lift) = box(width: 100%, height: 12pt)[
  #place(if lift { top } else { horizon }, line(length: 100%, stroke: 2pt))
  #if not lift { place(center + horizon, circle(radius: 4pt, fill: orange)) }
]
#grid(columns: (1fr,) * 7, row-gutter: 0pt,
  ..range(6).map(i => align(center)[*#(1817 + 20 * i)*]), [],
  ..range(6).map(_ => piece(false)), piece(true),
)
