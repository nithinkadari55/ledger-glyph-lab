// expect: L051.hyphenated-label
#set page(paper: "a4", margin: 2cm)
#set text(font: "IBM Plex Sans", size: 8.5pt, lang: "en", hyphenate: true)
#grid(
  columns: (56pt, 1fr),
  column-gutter: 8pt,
  align: horizon,
  [TX-100 · housing], rect(width: 100%, height: 10pt, fill: luma(200)),
)
