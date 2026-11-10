// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
Answers:

#grid(columns: (2em, 4em), row-gutter: 1.8em, column-gutter: 6pt, align: (right, center),
  [1.], $display(1/6)$,
  [2.], $display(4/15)$,
  [3.], $display(1/2)$,
  [4.], $display(3/10)$,
)
