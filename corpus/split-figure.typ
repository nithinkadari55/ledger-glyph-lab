// expect: L026.split-figure
#set page(paper: "a5", margin: 2cm)
#show figure: set block(breakable: true)
#figure(
  table(columns: 2, ..range(60).map(i => [Row #i]).map(c => (c, c)).flatten()),
  caption: [A long table],
)
