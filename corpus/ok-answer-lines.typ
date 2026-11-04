// expect-none
#set page(paper: "a5", margin: 2cm)
#set text(size: 11pt)
#let blank = box(width: 1fr, height: 1.1em, stroke: (bottom: 0.7pt))
#grid(columns: (1fr, 1fr), column-gutter: 20pt, row-gutter: 10pt, inset: (y: 4pt),
  ..range(8).map(i => [*#(i + 1).* $1/4 + 1/2 =$ #blank]))
