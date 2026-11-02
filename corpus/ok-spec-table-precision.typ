// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#table(
  columns: (1fr, auto, auto),
  align: (left, right, left),
  inset: 5pt,
  [*Parameter*], [*Max*], [*Unit*],
  [Process temperature], [125], [°C],
  [Accuracy], [0.5], [°C],
  [Resolution], [0.01], [°C],
  [Process pressure], [40], [bar],
)
