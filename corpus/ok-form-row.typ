// expect-none
#set page(paper: "a5", margin: 2cm)
#set text(size: 11pt)
#grid(columns: (auto, 1fr, auto, 4cm), column-gutter: 6pt, align: bottom,
  [Name:], line(length: 100%), [Date:], line(length: 100%))
