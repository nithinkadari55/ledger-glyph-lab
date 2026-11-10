// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#set par(spacing: 0.9em)
#text(16pt, weight: 700)[Harbour Point's rooftop array goes live]

#grid(
  columns: (1fr, 1fr),
  column-gutter: 12pt,
  [
    #lorem(40)

    #lorem(30)
  ],
  figure(rect(width: 100%, height: 120pt, fill: luma(220)), caption: [Artist's impression of the array.]),
)
