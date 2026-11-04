// expect-none
#set page(paper: "a6", margin: 1.5cm)
#set text(size: 11pt)
#block(below: 0pt, text(size: 22pt)[Photography gone])
#block(above: 8pt, line(length: 100%, stroke: 1.5pt))
#v(1em)
Some #underline[underlined gyp], #strike[struck], #highlight[highlighted gyp] and
#box(fill: luma(225), inset: (x: 2pt), outset: (y: 2pt))[boxed gyp] words.
