// expect: L034.top-gap
#set page(paper: "a5", margin: 2cm)
#set par(justify: false)
First page text.
#pagebreak()
Second page text.
#pagebreak()
#v(3cm)
Third page text starts too low.
#pagebreak()
Fourth page text.
