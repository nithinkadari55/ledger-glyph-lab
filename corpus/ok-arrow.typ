// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(
  columns: 3,
  column-gutter: 6pt,
  align: horizon + center,
  box(fill: luma(225), inset: 8pt)[Working directory],
  [→],
  box(fill: luma(225), inset: 8pt)[Staging area],
)
#lorem(60)
