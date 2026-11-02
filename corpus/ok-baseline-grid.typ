// expect-none
#set page(paper: "a5", margin: 2cm, baseline-grid: 15pt)
#set par(justify: false)
#set heading(numbering: "1.")

= Introduction
#lorem(60)

$ a^2 + b^2 = c^2 $

#lorem(40)

#figure(rect(width: 50%, height: 2.3cm), caption: [A box of arbitrary height])

== Details
#lorem(120)

#quote(block: true, attribution: [Someone])[#lorem(35)]

#block(inset: 7pt, stroke: 0.5pt, width: 100%)[#lorem(30)]

#lorem(50)
