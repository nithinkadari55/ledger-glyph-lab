// expect: L051.hyphenated-label
#set page(paper: "a4", margin: 2cm)
#set text(size: 9pt, lang: "en", hyphenate: true)
#grid(
  columns: (40pt, 1fr),
  column-gutter: 8pt,
  [Measurement], rect(width: 100%, height: 10pt, fill: luma(200)),
)
