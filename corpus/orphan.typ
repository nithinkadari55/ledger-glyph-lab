// expect: L010.orphan
#set page(paper: "a5", margin: 2cm)
#set text(costs: (orphan: 0%, widow: 0%))
#block(width: 100%, height: 455pt, fill: luma(240))
#lorem(40)
