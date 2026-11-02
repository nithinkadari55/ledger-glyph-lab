// expect: L058.malformed-number
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#table(
  columns: (1fr, auto),
  align: (left, right),
  [Subtotal], [€ 18,677.50],
  [Discount (5%)], [€ −93,3.88],
  [Net amount], [€ 17,743.62],
)

Version `1,2.3` in code stays literal.
