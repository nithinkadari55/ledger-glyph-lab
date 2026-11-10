// expect: L084.heading-crowded-above
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 11pt)
#let section(t) = block(above: 0pt, below: 0pt, align(center, stack(spacing: 6pt,
  text(font: "Cinzel", size: 11pt, tracking: 0.15em, upper(t)),
  line(length: 30pt, stroke: 0.6pt),
)))
#section[Readings]
#v(18pt)
Sonnet 116, read by Hannah Whitlock, and a passage chosen by the two families.
#v(9.5pt)
#section[The Wedding Party]
#v(18pt)
Maid of honour, best man, bridesmaids and groomsmen, named in order of entrance.
#v(9.5pt)
#section[With Thanks]
#v(18pt)
Thank you for being here with us today, and for every journey made to join us.
