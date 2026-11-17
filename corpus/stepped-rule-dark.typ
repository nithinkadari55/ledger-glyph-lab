// expect: L060.double-rule
#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt)
#block(width: 100%, fill: luma(232), inset: 14pt)[
  #block(width: 100%, stroke: (bottom: 1.2pt), inset: (bottom: 6pt), below: 10pt)[*HEADLINE ACTS*]
  #grid(columns: (1fr, 1fr, 1fr), column-gutter: 18pt,
    ..range(3).map(i => block(width: 100%, stroke: (top: 4pt), inset: (top: 12pt))[
      #text(size: 16pt, weight: "bold")[Act #(i + 1)]

      #lorem(18)
    ]))
]
