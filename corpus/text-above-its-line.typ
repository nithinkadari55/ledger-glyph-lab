// expect: L090.text-above-its-line
#set page(paper: "a5", flipped: true, margin: 1.5cm)
#set text(size: 11pt)
#align(center)[
  #text(size: 20pt, weight: "bold")[Certificate of Completion]

  Awarded for completing the course in applied statistics with distinction.
]
#v(1fr)
#grid(columns: (1fr, 1fr), column-gutter: 40pt,
  ..("Helena Voss", "Marcus Okafor").map(name => align(center)[
    #text(font: "Dancing Script", size: 22pt)[#name]
    #v(26pt, weak: true)
    #line(length: 70%, stroke: 0.7pt)
    #v(5pt, weak: true)
    #text(size: 10pt, weight: "bold")[#name]
  ]),
)
