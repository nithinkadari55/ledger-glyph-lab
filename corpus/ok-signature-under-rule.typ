// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#lorem(80)

#block(width: 100%, stroke: (bottom: 0.8pt), inset: (bottom: 4pt), below: 1.5em)[*Sign-off*]
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 22pt,
  ..("Form tutor", "Parent", "Headteacher").map(r => stack(spacing: 6pt,
    line(length: 100%, stroke: 0.6pt),
    text(size: 9pt)[#r · Signature and date])))
