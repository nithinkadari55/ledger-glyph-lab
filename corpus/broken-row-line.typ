// expect: L100.broken-row-line
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#table(columns: (auto, 1fr), column-gutter: 10pt, stroke: (x, y) => (bottom: 0.5pt), align: (left, right),
  [Weight], [1.65 kg], [Capacity], [2 person], [Waterproofing], [3,000 mm], [Packed size], [45 × 14 cm])
