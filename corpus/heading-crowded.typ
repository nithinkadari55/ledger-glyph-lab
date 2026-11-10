// expect: L084.heading-crowded-above
// expect: L038.cramped-lines
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 11pt)
#let section(t) = block(width: 100%, above: 1em, below: 1.2em, stroke: (bottom: 0.6pt), inset: (bottom: 5pt))[#text(12pt, tracking: 0.1em)[#t]]
#align(center, text(font: "Great Vibes", size: 30pt)[Eleanor & James])
#section[ORDER OF SERVICE]
2.00 pm Prelude, music as guests are seated.

#section[READINGS]
Sonnet 116, read by Hannah Whitlock.
