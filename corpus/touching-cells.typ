// expect: L040.touching-cells
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#align(right)[
  #block(width: 6cm)[
    #grid(
      columns: (1fr, auto),
      row-gutter: 6pt,
      [Subtotal], [1,200.00],
      [Tax (10%)], [120.00],
    )
  ]
]
