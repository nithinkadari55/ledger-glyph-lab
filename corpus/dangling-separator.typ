// expect: L062.dangling-separator
#set page(paper: "a4", margin: 2cm, columns: 2)
#set text(size: 10pt)
#set par(justify: false)
#lorem(60)
#colbreak()
#line(length: 100%, stroke: 0.6pt)
= Second article
#lorem(40)
