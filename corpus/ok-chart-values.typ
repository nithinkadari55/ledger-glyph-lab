// expect-none
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 9pt)
#grid(columns: (1fr,) * 5, column-gutter: 8pt, rows: 100pt, align: bottom,
  ..(40, 62, 55, 80, 92).map(v => rect(width: 100%, height: v * 1pt, fill: blue, inset: (top: 4pt),
    align(top + center, text(fill: white, weight: 700)[#v]))))
