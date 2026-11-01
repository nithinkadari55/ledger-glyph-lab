// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt, lang: "en")

#grid(
  columns: (1fr, auto),
  align: (left + top, right + top),
  [
    #text(18pt, weight: "bold")[Northwind Studio] \
    12 Harbour Road \
    Port City 4410
  ],
  [
    #text(14pt, weight: "bold")[Invoice] \
    No. 2026-0412 \
    Date: 1 November 2026
  ],
)

#v(1.5em)

*Bill to:* Example Trading Ltd., 88 Market Street, Riverton

#v(1em)

#table(
  columns: (auto, 1fr, auto, auto, auto),
  align: (center, left, right, right, right),
  inset: 6pt,
  fill: (_, y) => if y == 0 { luma(235) },
  table.header([*\#*], [*Description*], [*Qty*], [*Unit*], [*Amount*]),
  [1], [Brand identity workshop], [1], [1,200.00], [1,200.00],
  [2], [Logo design, three concepts with two revision rounds], [1], [2,400.00], [2,400.00],
  [3], [Stationery layout], [4], [150.00], [600.00],
  [4], [Website style guide], [1], [950.00], [950.00],
)

#align(right)[
  #table(
    columns: (auto, auto),
    align: (left, right),
    stroke: none,
    inset: 4pt,
    [Subtotal], [5,150.00],
    [Tax (10%)], [515.00],
    [*Total*], [*5,665.00*],
  )
]

#v(1fr)

Payment is due within 30 days. Please quote the invoice number with your
payment. Thank you for your business.
