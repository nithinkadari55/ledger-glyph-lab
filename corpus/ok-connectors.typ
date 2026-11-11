// expect-none
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#let node(body) = rect(width: 100%, height: 30pt, fill: luma(232), stroke: 0.6pt, align(center + horizon, body))
#let drop = block(width: 100%, height: 14pt, line(start: (50%, 0pt), end: (50%, 14pt), stroke: 1pt))
#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 12pt,
  grid.cell(colspan: 3, align(center, box(width: 40%, node[Studio director]))),
  grid.cell(colspan: 3, drop),
  grid.cell(colspan: 3, pad(x: (100% - 24pt) / 6, line(length: 100%, stroke: 1pt))),
  drop, drop, drop,
  node[Brand lead], node[Product lead], node[Motion lead],
)
