// expect-none
#set page(paper: "a6", margin: 1cm)
#set text(size: 9pt)
#align(center, box(width: 60%, stroke: 0.8pt, inset: 6pt)[Director])
#grid(
  columns: (1fr, 1fr, 1fr),
  rows: (14pt, 14pt),
  grid.cell(colspan: 3, block(width: 100%, height: 100%, align(bottom, line(length: 100%, stroke: 1.4pt)))),
  ..range(3).map(_ => block(width: 100%, height: 100%, align(center, line(angle: 90deg, length: 100%, stroke: 1.4pt)))),
)
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 6pt,
  ..range(3).map(i => box(width: 100%, stroke: 0.8pt, inset: 6pt)[Lead #(i + 1)]))
