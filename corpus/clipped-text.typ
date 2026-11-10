// expect: L082.clipped-text
#set page(paper: "a6", margin: 1cm)
#set text(size: 9pt)
#block(width: 110pt, radius: 6pt, clip: true, fill: luma(235), inset: 8pt)[
  #stack(spacing: 14pt, text(8pt, weight: 700)[ADMIT ONE], text(size: 26pt)[RHP025431])
]
