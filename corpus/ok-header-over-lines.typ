// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#lorem(30)
#v(1em)
#block(width: 100%, stroke: (bottom: 0.8pt + green), inset: (bottom: 4pt), below: 1.4em)[*Notes*]
#grid(columns: 1fr, row-gutter: 2.4em,
  ..range(4).map(_ => block(width: 100%, height: 1pt, stroke: (bottom: 0.5pt + luma(170)))))
