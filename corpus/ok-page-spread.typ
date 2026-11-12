// expect-none
#set page(paper: "a5", margin: 1.8cm)
#set text(size: 10pt)
#let section(title, body) = [
  #block(below: 0.8em, text(weight: "bold", title))
  #body
]
#align(center, text(size: 20pt, weight: "bold")[Kemuri])
#v(14pt)
#v(1fr)
#section[Small plates][Edamame with sea salt \ Gyoza with ponzu \ Karaage with yuzu mayonnaise]
#v(14pt)
#v(1fr)
#section[Grills][Yakitori with spring onion \ Salmon belly with miso \ Aubergine with sesame]
#v(14pt)
#v(1fr)
#section[Desserts][Matcha cheesecake \ Hojicha pudding \ Mochi ice cream]
#v(14pt)
#v(1fr)
#text(size: 8pt)[Please tell your server about any allergy before ordering.]
