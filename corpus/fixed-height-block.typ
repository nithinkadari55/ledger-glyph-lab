// expect: L002.container-overflow
#set page(paper: "a4", margin: 2cm)
#block(height: 2cm, width: 100%, stroke: 0.5pt, breakable: false)[
  #lorem(200)
]
