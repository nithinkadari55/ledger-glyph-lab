// expect: L005.loose-line
#show figure: set block(above: 2em, below: 2em)
// expect: L044.centered-paragraph
#set page(paper: "a4", margin: (x: 1.8cm, y: 2.2cm), columns: 2)
#set columns(gutter: 1.2em)
#set text(size: 10pt, lang: "en")
#set par(justify: true)
#set heading(numbering: "1.")

#place(top + center, float: true, scope: "parent")[
  #align(center)[
    #text(17pt, weight: "bold")[Measuring Urban Heat Islands with Low-Cost Sensors]
    #v(0.3em)
    #text(10pt)[Field Methods Group]
  ]
  #v(0.6em)
  #block(width: 100%, inset: 8pt, fill: luma(245))[
    *Abstract.* We deployed sixty low-cost temperature sensors across a mid-sized
    city for one summer and compared them with reference stations. After a
    simple calibration, the network resolved neighbourhood-scale differences of
    up to four degrees between dense and green districts.
  ]
  #v(0.6em)
]

= Introduction
#lorem(110)

= Method
#lorem(90)

#figure(
  rect(width: 100%, height: 3.2cm, fill: luma(230)),
  caption: [Sensor placement across the study area.],
)

#lorem(80)

= Results
#lorem(140)

= Discussion
#lorem(120)
