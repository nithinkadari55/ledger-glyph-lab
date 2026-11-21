// expect: L102.broken-axis
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#block(width: 100%)[
  #grid(columns: (1fr,) * 5, rows: (130pt, 12pt),
    ..range(5).map(i => grid.cell(stroke: (left: 1pt + orange), inset: (left: 6pt), align: bottom)[*#(1817 + 20 * i)* \ A step in the history of the bicycle]),
    ..range(5).map(_ => box(width: 100%, height: 12pt)[#place(horizon, line(length: 100%, stroke: 2pt))#place(left + horizon, circle(radius: 4pt, fill: orange))]),
  )
]
