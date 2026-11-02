// expect: L039.hollow-box
// expect: L025.near-empty-last-page
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
= Key features
The sensor family shares one housing and one firmware, so the features below
apply to every model in the range.

#grid(
  columns: (1fr, 1fr),
  gutter: 8pt,
  block(width: 100%, height: 100%, inset: 8pt, fill: luma(240), stroke: 0.5pt)[
    *Accurate.* Typical error below 0.2 degrees.
  ],
  block(width: 100%, height: 100%, inset: 8pt, fill: luma(240), stroke: 0.5pt)[
    *Durable.* Sealed stainless housing.
  ],
)
