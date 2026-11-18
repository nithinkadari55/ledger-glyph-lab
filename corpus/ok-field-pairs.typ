// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#grid(columns: (1fr, 1fr), column-gutter: 18pt, row-gutter: 10pt, inset: (bottom: 14pt),
  stroke: (x, y) => (bottom: 0.8pt),
  text(size: 8pt, weight: 700)[ROUTE], text(size: 8pt, weight: 700)[DATES],
  text(size: 8pt, weight: 700)[HIKERS], text(size: 8pt, weight: 700)[PACK WEIGHT GOAL],
)
