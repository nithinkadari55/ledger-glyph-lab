// expect-none
#set page(paper: "a5", margin: 2cm)
#set text(size: 11pt)
#grid(columns: (1fr,) * 3, column-gutter: 20pt, row-gutter: 14pt,
  ..range(9).map(i => [*#(i + 1).* $1/4 + 1/2 = 3/4$]))
