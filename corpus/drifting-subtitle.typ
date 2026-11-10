// expect: L087.drifting-subtitle
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#lorem(120)

#block(fill: luma(240), inset: 12pt, width: 50%)[
  #block(below: 32pt, text(18pt, weight: 700)[11:00])
  #text(12pt, weight: 700)[Reading the River]

  #text(10pt)[A walk along the estuary.]
]
