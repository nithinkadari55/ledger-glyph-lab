// expect: L084.heading-crowded-above
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
#set par(spacing: 1.1em)
#let role(title, place, body) = block(above: 1.1em, below: 0pt)[
  *#title* #h(4pt) #text(fill: luma(90))[#place]
  #v(3pt)
  #body
]
#role[Lead Designer][Northbank, London][
  - Led a team of six designers on the mobile banking app.
  - Introduced fortnightly interviews with customers.
]
#role[Senior Product Designer][Helix Health, London][
  - Designed a booking platform used by three hundred clinics.
  - Ran usability studies and accessibility audits.
]
#role[Designer][Marlow and Pine, Bristol][
  - Delivered web and mobile projects for retail clients.
  - Mentored two junior designers in the studio.
]
