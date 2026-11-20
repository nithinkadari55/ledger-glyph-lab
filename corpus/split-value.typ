// expect: L101.split-value
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10.5pt)
#block(fill: luma(240), inset: 10pt, width: 100%)[
  #grid(columns: (1fr, 1fr, 1fr), column-gutter: 12pt,
    [#text(size: 8pt, weight: 700)[DATE] \ Tuesday, 14 October 2025],
    [#text(size: 8pt, weight: 700)[TIME] \ 10:00 – 11:30],
    [#text(size: 8pt, weight: 700)[ROOM] \ Meeting Room 4B],
  )
]
