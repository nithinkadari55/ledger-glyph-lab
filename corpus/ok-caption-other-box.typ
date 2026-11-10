// expect-none
#set page(paper: "a6", margin: 1cm)
#set text(size: 9pt)
#let ticket = block(width: 100%, height: 110pt, fill: luma(235), inset: 10pt)[
  #text(8pt, weight: 700)[ADMIT ONE]
  #v(2pt)
  #text(24pt)[RHP025431]
]
#ticket
#v(40pt)
#ticket
