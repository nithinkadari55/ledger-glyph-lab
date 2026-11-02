// expect: L061.tight-box-padding
#set page(paper: "a4", margin: 2cm)
#block(width: 100%, fill: rgb("#1e2a3a"), inset: (top: 2pt, bottom: 14pt, x: 12pt))[
  #text(30pt, weight: "bold", fill: white)[The Northwind Dispatch]
  #v(8pt)
  #text(9pt, fill: rgb("#f4b183"))[NEWS, PEOPLE AND PLANS]
]
