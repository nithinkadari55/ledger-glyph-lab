// expect: L042.hyphenated-display-text
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt, lang: "en")
#block(width: 5cm)[
  #set text(size: 18pt, hyphenate: true)
  Quarterly internationalization results
]
#v(14pt)

The quarter closed with growth in every region.
