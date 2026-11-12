// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#block(width: 100%, stroke: (bottom: 1pt), inset: (bottom: 4pt), below: 10pt)[*Meeting details*]
#table(
  columns: (1fr, 2fr),
  inset: 6pt,
  fill: (x, y) => if y == 0 { luma(230) },
  [*Date*], [Friday, 14 March 2025],
  [*Time*], [10:00 to 11:30],
)
