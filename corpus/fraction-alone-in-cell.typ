// expect: L085.cramped-fraction-rows
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 11pt)
Each answer below is given in its simplest form, so check that your own answers have been simplified before you mark them.
#block(fill: luma(245), inset: 10pt, width: 100%)[
  #grid(columns: (auto, 1fr, auto, 1fr, auto, 1fr), column-gutter: 6pt, row-gutter: 0.8em,
    align: (right + horizon, left + horizon),
    ..($1/6$, $1/2$, $3/20$, $1/3$, $1/2$, $1/7$).enumerate().map(((i, k)) =>
      ([#(i + 1).], k)).flatten(),
  )
]
