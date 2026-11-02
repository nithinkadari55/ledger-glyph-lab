// expect: L047.low-contrast
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#table(
  columns: (1fr, 1fr),
  fill: (_, y) => if y == 0 { rgb("#e8f0f5") },
  table.header(text(fill: white, weight: "bold")[Document details], []),
  [Prepared by], [Digital Transformation Office],
  [Version], [1.0 (for approval)],
)
