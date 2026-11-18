// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#let sub(body) = pad(left: 10pt, body)
#table(columns: (1fr, auto, auto), stroke: (x, y) => (bottom: 0.4pt), align: (left, right, right),
  [], [*Per 100 g*], [*Per serving*],
  [Energy (kJ)], [1840], [828],
  [Fat (g)], [17.0], [7.7],
  sub[of which saturates (g)], [3.1], [1.4],
  [Carbohydrate (g)], [59.0], [26.6],
  sub[of which sugars (g)], [14.0], [6.3],
  [Fibre (g)], [8.5], [3.8],
  [Protein (g)], [10.5], [4.7],
)
