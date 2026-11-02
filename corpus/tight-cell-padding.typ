// expect: L059.tight-cell-padding
#set page(paper: "a4", margin: 2cm)
#set text(size: 9pt)
#table(
  columns: (1fr, auto),
  stroke: (x, y) => (bottom: 0.5pt + luma(180)),
  inset: (x: 5pt, y: 3.5pt),
  [*Brand discovery workshop* \ #text(fill: luma(110))[Two-day on-site session with leadership]], [1,900.00],
  [*Research synthesis* \ #text(fill: luma(110))[Stakeholder interviews and market review]], [1,200.00],
)
