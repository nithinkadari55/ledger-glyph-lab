// expect: L007.page-underfill
#set page(paper: "a5", margin: 2cm)
#lorem(150)

#block(breakable: false, width: 100%, height: 13cm, inset: 8pt, fill: luma(235))[A tall unbreakable panel] <allow-empty>

#lorem(40)
