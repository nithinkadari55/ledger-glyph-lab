// expect: L032.column-overflow
// expect: L043.empty-column
#set page(paper: "a4", margin: 2cm, columns: 2)
#lorem(40)

#box(width: 11cm, height: 1cm, fill: luma(220))

#lorem(40)
