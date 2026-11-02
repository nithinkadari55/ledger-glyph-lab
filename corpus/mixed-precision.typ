// expect: L041.mixed-precision
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#table(
  columns: (1fr, auto, auto),
  align: (left, right, right),
  inset: 5pt,
  [*Cost item*], [*Amount*], [*Share*],
  [Cloud infrastructure], [185,000], [23%],
  [Migration services], [240,000], [29.9%],
  [Security testing], [95,000], [11.8%],
  [Project management], [80,000], [10%],
)
