// expect-none
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#let card(year, body) = rect(width: 100%, fill: luma(235), inset: 7pt)[*#year* \ #body]
#let stem = block(width: 100%, height: 14pt, line(start: (50%, 0pt), end: (50%, 14pt), stroke: 1pt))
#grid(
  columns: (1fr, 1fr, 1fr, 1fr),
  column-gutter: 10pt,
  align: bottom,
  card[1817][The running machine is shown in Mannheim and carries its rider on two wheels.],
  card[1866][Pedals are fixed to the front wheel.],
  card[1885][The safety bicycle puts two equal wheels under a chain drive, the layout still in use.],
  card[1888][Air-filled tyres smooth the ride.],
  stem, stem, stem, stem,
  grid.cell(colspan: 4, line(length: 100%, stroke: 1pt)),
  grid.cell(colspan: 4, v(6pt)),
  [Mannheim], [Paris], [Coventry], [Belfast],
)
