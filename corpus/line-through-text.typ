// expect: L050.line-through-text
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#box(width: 6cm, height: 2cm)[
  #place(top + left, dy: 20pt)[Bar label 112]
  #place(top + left, dy: 23.5pt, line(length: 100%, stroke: 0.5pt + luma(150)))
]
