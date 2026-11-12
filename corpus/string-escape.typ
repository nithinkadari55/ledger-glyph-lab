// expect: S006.string-escape
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#let kpis = (("Revenue", "\$84.6 M"), ("Cost per consignment", "\$56.9"))
#for (name, value) in kpis [
  #name: #value \
]
#lorem(60)
