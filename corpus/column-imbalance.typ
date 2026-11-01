// expect: L019.column-imbalance
#set page(paper: "a5", margin: 2cm)
#set par(justify: false)
#columns(2)[#lorem(170)]
#pagebreak()
Next part.
