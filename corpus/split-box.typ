// expect: L081.split-box
#set page(paper: "a6", margin: 1.5cm)
#set text(size: 10pt)
#lorem(160)

#block(width: 100%, fill: luma(235), inset: 8pt)[
  #set par(spacing: 0.6em)
  *Tips*

  - Tip one is short.
  - Tip two is short.
  - Tip three is short.
  - Tip four is short.
  - Tip five is short.
]
