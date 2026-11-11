// expect-none
#set page(paper: "a4", margin: 2.2cm)
#set text(size: 11pt)
#text(size: 18pt, weight: "bold")[Letter of appointment]

#lorem(90)

#lorem(70)

#lorem(60)

#v(1fr)
#grid(
  columns: (1fr, 1fr),
  column-gutter: 40pt,
  [#line(length: 100%, stroke: 0.6pt) #v(4pt, weak: true) Head of department],
  [#line(length: 100%, stroke: 0.6pt) #v(4pt, weak: true) Date],
)
