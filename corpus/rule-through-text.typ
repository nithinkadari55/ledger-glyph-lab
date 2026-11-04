// expect: L050.line-through-text
#set page(paper: "a6", margin: 1.5cm)
#set text(size: 11pt)
#block(below: 0pt, text(size: 22pt)[Photography gone])
#block(above: 0pt, line(length: 100%, stroke: 1.5pt))
#v(1em)
The rule above runs through the descenders of the title.
