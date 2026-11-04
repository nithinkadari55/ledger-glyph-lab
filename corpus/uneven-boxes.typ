// expect: L069.uneven-boxes
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(
  columns: (1fr, 1.5fr),
  column-gutter: 12pt,
  block(width: 100%, fill: rgb("#e8eef5"), inset: 10pt)[*Progress* \ 3 of 16 tasks done],
  block(width: 100%, fill: rgb("#e8eef5"), inset: 10pt)[*Legend* \ Empty box: open \ Ticked box: finished \ Three stars: critical \ One star: nice to have],
)
#lorem(60)
