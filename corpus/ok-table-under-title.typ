// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#text(size: 20pt, weight: 700)[What's on this season]
#v(4pt)
#figure(
  table(columns: (auto, 1fr, auto), stroke: none, inset: 4pt, align: left,
    [*Date*], [*Event*], [*Time*],
    [8 April], [Quarterly all-hands], [10:00],
    [17 April], [Open house], [14:00],
    [24 April], [First-aid course], [09:30],
  ),
  caption: [Dates for the coming quarter.],
)
