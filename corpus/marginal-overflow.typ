// expect: L018.marginal-overflow
// expect: L038.cramped-lines
// expect: L067.print-edge
#set page(
  paper: "a5",
  margin: (top: 1.5cm, rest: 2cm),
  header: [#text(24pt)[Quarterly] \ #text(24pt)[Report] \ #text(24pt)[Internal]],
)
#lorem(40)
