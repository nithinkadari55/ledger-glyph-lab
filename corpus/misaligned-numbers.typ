// expect: L017.misaligned-numbers
#set page(paper: "a4", margin: 2cm)
#table(
  columns: (auto, 3cm),
  [Item], [Amount],
  [Rent], [1,250.00],
  [Food], [86.40],
  [Travel], [3.15],
  [Other], [12,400.00],
)
