// expect: L064.detached-subtitle
// expect: L071.detached-label
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#set par(spacing: 1em)
#align(center)[
  #text(size: 9pt, tracking: 0.2em)[DINNER MENU]
  #v(8pt)
  #text(size: 40pt, weight: "bold")[Tsukimi]
  #v(10pt)
  #text(size: 11pt, style: "italic")[Small plates, charcoal grills and sake]
]

#lorem(80)
