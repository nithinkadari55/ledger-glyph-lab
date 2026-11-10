// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#grid(columns: (1fr, 1fr, 1fr), row-gutter: 1em, column-gutter: 10pt,
  [#text(8pt)[DATE] \ Tue 14 May], [#text(8pt)[TIME] \ 10:00], [#text(8pt)[ROOM] \ 4B],
  [#text(8pt)[HOST] \ Priya Nair], [#text(8pt)[NOTES] \ None], [#text(8pt)[CODE] \ X1],
)
