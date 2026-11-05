// expect-none
#set page(paper: "a6", margin: 1.2cm)
#set text(size: 10pt)
#table(
  columns: (auto, 1fr, 1fr),
  stroke: (_, y) => (bottom: 0.5pt + luma(200)),
  inset: (x: 6pt, y: 5pt),
  [Typical values], table.cell(align: right)[per 100 g], table.cell(align: right)[per serving],
  [Energy (kJ)], table.cell(align: right)[1920], table.cell(align: right)[864],
  [Fat (g)], table.cell(align: right)[18.4], table.cell(align: right)[8.3],
  table.cell(inset: (left: 20pt, right: 6pt, y: 5pt))[of which saturates (g)], table.cell(align: right)[3.1], table.cell(align: right)[1.4],
  [Carbohydrate (g)], table.cell(align: right)[57.2], table.cell(align: right)[25.7],
  table.cell(inset: (left: 20pt, right: 6pt, y: 5pt))[of which sugars (g)], table.cell(align: right)[14.6], table.cell(align: right)[6.6],
)<kit-spec-table>
