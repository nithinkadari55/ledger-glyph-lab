// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
= Staff spotlight
#columns(2, gutter: 20pt, balanced: true)[
  *How did you start?* #lorem(40)

  *What are you proudest of?* #lorem(40)
]
#v(2em)
#lorem(60)
