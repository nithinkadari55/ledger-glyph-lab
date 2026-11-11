// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#grid(columns: (1fr, 1fr), column-gutter: 12pt, row-gutter: 12pt,
  block(width: 100%, stroke: (left: 3pt + navy, rest: 0.7pt + gray), inset: 8pt)[Design lead],
  block(width: 100%, stroke: 4pt + gray, inset: 8pt)[Research lead],
  grid.cell(stroke: (bottom: 1.2pt))[],
  [Reports to the director],
)
