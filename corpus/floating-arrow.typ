// expect: L066.floating-arrow
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(
  columns: (1fr, 1fr, 1fr),
  align: horizon + center,
  box(fill: luma(225), inset: 8pt)[Working directory],
  [→],
  box(fill: luma(225), inset: 8pt)[Staging area],
)
#lorem(60)
