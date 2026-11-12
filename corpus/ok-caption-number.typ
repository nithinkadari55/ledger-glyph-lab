// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#lorem(60)

#figure(
  table(
    columns: (auto, 1fr, 1fr),
    align: (left, right, right),
    table.header([*Region*], [*Q1*], [*Q2*]),
    [North], [1,204], [1,388],
    [South], [988], [1,021],
  ),
  caption: [Quarterly figures by region; Table 2 gives the yearly totals.],
)

#figure(
  table(
    columns: (auto, 1fr, 1fr),
    align: (left, right, right),
    table.header([*Region*], [*Q3*], [*Q4*]),
    [North], [1,502], [1,610],
    [South], [1,130], [1,214],
  ),
  numbering: none,
  caption: [*Table 2.* Second-half figures by region.],
)

#lorem(60)
