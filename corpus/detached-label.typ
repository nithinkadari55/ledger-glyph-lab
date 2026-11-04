// expect: L071.detached-label
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#block(below: 60pt, text(8.5pt, weight: "bold", tracking: 0.1em)[FEATURE · GREEN CITIES])
#block(above: 0pt, below: 10pt, text(40pt, weight: "bold")[The Sky Is the New Soil])
#block(above: 0pt, below: 18pt, text(13pt, style: "italic")[From Brooklyn to Berlin, forgotten roofs are turning into farms.])
#lorem(120)
