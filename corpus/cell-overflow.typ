// expect: L002.container-overflow
// expect: L035.row-baselines
#set page(paper: "a4", margin: 2cm)
#table(
  columns: (3cm, 1fr),
  [#box(width: 5cm, height: 1em, fill: gray)], [Second column],
)
