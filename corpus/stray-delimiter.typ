// expect: S007.stray-delimiter
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 12pt,
  [#lorem(30)],
  [#lorem(30)],
)
)
#lorem(60)
