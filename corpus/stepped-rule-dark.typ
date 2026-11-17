// expect: L060.double-rule
#set page(paper: "a4", margin: 2cm, fill: rgb("#0f1b2d"))
#set text(size: 11pt, fill: rgb("#f4ecd8"))
#block(width: 100%, stroke: (bottom: 1.2pt + rgb("#f2a900")), inset: (bottom: 6pt), below: 10pt)[HEADLINE ACTS]
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 18pt,
  ..range(3).map(i => block(width: 100%, stroke: (top: 4pt + rgb("#4cc9b8")), inset: (top: 12pt))[
    #text(size: 16pt, weight: "bold")[Act #(i + 1)]

    #lorem(18)
  ]))
