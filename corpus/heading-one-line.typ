// expect: L014.stranded-heading
#set page(paper: "a5", margin: 2cm)
#set text(costs: (orphan: 0%, widow: 0%))
#show heading: set block(sticky: false)
#block(width: 100%, height: 425pt, fill: luma(240))
= Results
#lorem(60)
