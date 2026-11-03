// expect: L063.chart-labels
#set page(paper: "a4", margin: 2cm)
#box(width: 10cm, height: 4.5cm)[
  #for (i, h) in ((0, 2cm), (1, 2.6cm), (2, 3cm)) {
    place(top + left, dy: 3cm - h, dx: 1cm + i * 3cm, rect(width: 1cm, height: h, fill: teal, stroke: none))
    place(top + left, dy: 3.9cm, dx: 0.6cm + i * 3cm, text(8pt)[M#(i + 1)])
  }
]
