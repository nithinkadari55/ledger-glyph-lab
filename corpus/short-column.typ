// expect: L091.short-column
#set page(paper: "a5", flipped: true, margin: 1.2cm)
#set text(size: 9pt)
#let para = [Sediments take up particles from treated effluent, road runoff and degraded litter, and once buried the particles can enter the food web through the animals that live in the river bed.]
#let column(n) = range(n).map(_ => para).join(parbreak())
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 16pt,
  column(4), column(4), column(2),
)
