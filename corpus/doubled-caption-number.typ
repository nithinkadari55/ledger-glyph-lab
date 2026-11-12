// expect: L094.doubled-caption-number
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#show figure: set block(spacing: 2.2em)
#lorem(60)

#figure(
  table(
    columns: (auto, 1fr, 1fr),
    align: (left, right, right),
    table.header([*Region*], [*Q1*], [*Q2*]),
    [North], [1,204], [1,388],
    [South], [988], [1,021],
  ),
  caption: [*Table 1.* Quarterly figures by region.],
)

#lorem(60)
