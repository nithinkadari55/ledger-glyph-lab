// expect: L085.cramped-fraction-rows
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#let f(a, b) = $display(#a / #b)$
Answers:

#grid(columns: (1fr, 1fr, 1fr, 1fr), column-gutter: 10pt, row-gutter: 1.2em,
  [*1.* #f(1, 6)], [*2.* #f(2, 15)], f(3, 8), f(3, 10),
  [*5.* #f(2, 5)], [*6.* #f(2, 7)], f(1, 4), f(1, 6),
)
