// expect: L057.sparse-bar-chart
#set page(paper: "a4", margin: 2cm)
#box(width: 12cm, height: 3cm)[
  #place(bottom + left, line(length: 100%, stroke: 0.5pt))
  #for (i, h) in ((0, 2cm), (1, 2.2cm), (2, 2.4cm)) {
    place(bottom + left, dx: 1cm + i * 4cm, rect(width: 0.5cm, height: h, fill: blue, stroke: none))
    place(bottom + left, dx: 1.55cm + i * 4cm, rect(width: 0.5cm, height: h - 0.3cm, fill: gray, stroke: none))
  }
]
