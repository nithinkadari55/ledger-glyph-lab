// expect: L074.detached-figure-caption
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#grid(columns: (1fr, 1fr, 1fr), column-gutter: 12pt,
  [#block(below: 3pt, text(24pt, weight: "bold")[45 dB]) #text(9pt)[Peak noise reduction]],
  [#block(below: 3pt, text(24pt, weight: "bold")[60 h]) #text(9pt)[Playback with ANC off]],
  [#block(below: 3pt, text(24pt, weight: "bold")[40 mm]) #text(9pt)[Driver size]],
)
#lorem(80)
