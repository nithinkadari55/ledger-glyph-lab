// expect: L065.detached-underline
#set page(paper: "a4", flipped: true, margin: 2cm)
#set text(size: 12pt)
#align(center)[
  This is to certify that
  #v(8pt)
  #text(40pt)[Eleanor Vance]
  #v(30pt)
  #line(length: 7cm, stroke: 0.6pt)
  #v(2pt, weak: true)
  has successfully completed the course in advanced data analysis.
]
#lorem(40)
