// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#let panel(title, n) = block(width: 100%, fill: luma(235), inset: 8pt, radius: 3pt)[
  *#title* \ #for i in range(n) [Item number #(i + 1) of this section \ ]
]
#grid(columns: (1fr, 1fr), column-gutter: 12pt,
  stack(spacing: 12pt, panel("Clothing", 9), panel("Footwear", 4)),
  stack(spacing: 12pt, panel("Food", 4), panel("Cooking", 5), panel("Safety", 3)),
)
