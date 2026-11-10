// expect: L085.cramped-fraction-rows
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#block(fill: luma(240), inset: 10pt, width: 100%)[
  #grid(columns: (1fr, 1fr, 1fr, 1fr), column-gutter: 10pt, row-gutter: 0.9em,
    [*1.* $1/6$], [*2.* $2/15$], [*3.* $3/8$], [*4.* $6/20 = 3/10$],
    [*5.* $6/15 = 2/5$], [*6.* $4/14 = 2/7$], [*7.* $15/60 = 1/4$], [*8.* $6/36 = 1/6$],
  )
]
