// expect: L039.hollow-box
#set page(paper: "a4", margin: 2cm, columns: 2)
#set text(size: 10pt)
#lorem(80)
#block(breakable: false, width: 100%, fill: luma(240))[
  #grid(
    columns: (3pt, 1fr),
    rect(width: 100%, height: 100%, fill: red, stroke: none),
    block(inset: 8pt)[#text(14pt, style: "italic")[“We built a place people will want to come back to.”]],
  )
]
#lorem(60)
