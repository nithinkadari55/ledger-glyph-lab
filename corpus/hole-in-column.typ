// expect: L015.inconsistent-spacing
// expect: L093.hole-in-column
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 16pt,
  rows: 420pt,
  [
    *Introduction*

    #lorem(40)

    #lorem(30)

    #v(1fr)
    *Methods*

    #lorem(40)

    #lorem(25)
  ],
  [
    *Results*

    #lorem(55)

    #lorem(50)

    *Conclusions*

    #lorem(50)

    #lorem(35)
  ],
)
#lorem(120)
