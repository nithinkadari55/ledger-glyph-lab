// expect: L043.empty-column
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
= Staff spotlight
#columns(2, gutter: 20pt)[
  *How did you start?* #lorem(30)

  *What are you proudest of?* #lorem(30)
]
#block(fill: luma(240), inset: 14pt, width: 100%)[
  *In brief*
  #columns(2, gutter: 20pt, balanced: true)[#lorem(70)]
]
