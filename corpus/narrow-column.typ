// expect: L029.narrow-column
#set page(paper: "a5", margin: 2cm)
#table(
  columns: (2cm, 1fr),
  [budgets planning staffing reports reviews approvals], [Owner],
)
