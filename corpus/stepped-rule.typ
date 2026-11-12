// expect: L060.double-rule
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#block(width: 100%, stroke: (bottom: 1pt), inset: (bottom: 4pt), below: 10pt)[*Key features*]
#grid(
  columns: (1fr, 1fr),
  column-gutter: 20pt,
  block(width: 100%, stroke: (top: 2pt), inset: (top: 8pt))[
    Three-stage filtration keeps the air in the room clean all day.
  ],
  block(width: 100%, stroke: (top: 2pt), inset: (top: 8pt))[
    Real-time sensing reads the air every second and adjusts the fan.
  ],
)
