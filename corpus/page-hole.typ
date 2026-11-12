// expect: L015.inconsistent-spacing
// expect: L093.hole-in-column
#set page(paper: "a5", margin: 1.8cm)
#set text(size: 10pt)
#align(center)[
  #text(size: 22pt, weight: "bold")[Eleanor and James]

  Saturday, 14 June 2026

  St Michael's Church, Hartley Green
]
#v(1fr)
#align(center)[◆ ◆ ◆]
#v(1fr)
#align(center)[*Order of service*]
#grid(
  columns: (auto, 1fr),
  column-gutter: 14pt,
  row-gutter: 0.75em,
  [Prelude], [Music as our guests are seated],
  [Processional], [The bride enters with her father],
  [Welcome], [Reverend Anna Whitlock],
  [Hymn], [Love Divine, All Loves Excelling],
  [First reading], [Read by Margaret Ashby],
  [Address], [Reverend Anna Whitlock],
  [Vows and rings], [The exchange of promises and rings],
  [Blessing], [Prayers and the blessing of the marriage],
  [Recessional], [Confetti on the church steps],
)
