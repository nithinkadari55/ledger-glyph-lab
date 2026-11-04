// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 12pt,
  [#block(below: 11pt, text(22pt, weight: "bold")[60 h]) #text(8.5pt)[Playback with ANC off]],
  [#block(below: 11pt, text(22pt, weight: "bold")[5 min]) #text(8.5pt)[Quick charge for 5 hours]],
  [#block(below: 11pt, text(22pt, weight: "bold")[254 g]) #text(8.5pt)[Weight, fully equipped]],
)
#lorem(80)
