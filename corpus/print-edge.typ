// expect: L067.print-edge
#set page(
  paper: "a5",
  margin: 2cm,
  background: place(top + left, dx: 8pt, dy: 8pt, rect(width: 100% - 16pt, height: 100% - 16pt, stroke: 1pt)),
)
#lorem(60)
