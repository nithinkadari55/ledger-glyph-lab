// expect: L102.broken-axis
// Round 48's timeline: each event line is a line down the event cell plus a
// short line in the axis row that reaches a 16.5pt dot. Only the two pieces
// joined reach the dot, and the line runs far above the text it marks.
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#let axh = 40pt
#grid(columns: (1fr,) * 4, rows: (130pt, axh),
  ..range(4).map(i => box(width: 100%, height: 130pt)[
    #place(dx: 10pt, line(angle: 90deg, length: 130pt, stroke: 1.2pt))
    #place(bottom + left, dx: 16pt, block(width: 80%)[*#(1817 + 20 * i)* \ A step in the history of the bicycle])
  ]),
  ..range(4).map(_ => box(width: 100%, height: axh)[
    #place(dy: axh / 2 - 1pt, line(length: 100%, stroke: 2pt))
    #place(dx: 10pt, line(angle: 90deg, length: axh / 2, stroke: 1.2pt))
    #place(dx: 10pt - 8.25pt, dy: axh / 2 - 8.25pt, circle(radius: 7.5pt, fill: red, stroke: 1.5pt))
  ]),
)
