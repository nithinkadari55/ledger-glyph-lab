// expect: L048.ragged-column
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#table(
  columns: (auto, 1fr),
  align: center,
  [*Phase*], [*Outcome*],
  [1 Discovery], [Inventory signed off],
  [2 Design], [Design approval],
  [3 Data and tooling], [Tooling ready],
  [4 Validation and handover], [Project closed],
)
