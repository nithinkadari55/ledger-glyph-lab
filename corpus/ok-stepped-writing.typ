// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#block(width: 100%, stroke: (bottom: 1pt), inset: (bottom: 4pt), below: 12pt)[*Notes*]
#for _ in range(4) {
  block(width: 70%, height: 18pt, stroke: (bottom: 0.5pt))
}
