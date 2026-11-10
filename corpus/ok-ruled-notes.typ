// expect-none
#set page(paper: "a6", margin: 1.2cm)
#set text(size: 10pt)
#block(width: 100%, stroke: (bottom: 0.8pt), inset: (bottom: 4pt), below: 0.4em)[*NOTES*]
#grid(
  columns: 1fr,
  ..range(4).map(_ => block(width: 100%, height: 20pt, stroke: (bottom: 0.6pt + luma(160)))),
)
