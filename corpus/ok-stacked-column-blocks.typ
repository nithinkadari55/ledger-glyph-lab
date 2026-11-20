// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt, hyphenate: true)
= Expenses now take two minutes
#columns(2, gutter: 8mm, balanced: true)[#lorem(140)]
= Staff spotlight
#block(inset: (x: 8mm))[#columns(2, gutter: 8mm, balanced: true)[#lorem(260)]]
