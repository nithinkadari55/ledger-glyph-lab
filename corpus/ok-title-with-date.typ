// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(size: 11pt)
#text(size: 40pt, weight: "bold")[Festival]

#grid(columns: (1fr, 1.2fr), column-gutter: 20pt, row-gutter: 36pt, align: horizon,
  ..(
    ("Nadia Okafor Quartet", "Fri 13 June, The Blue Anchor", "A pianist whose groove-heavy pieces have reshaped modern soul jazz."),
    ("Marcus Delaney Band", "Thu 12 June, Harbour Hall", "Seventeen players and one roaring sound in fearless new arrangements."),
    ("Sofia Marchetti", "Sat 14 June, Granary Stage", "A vocalist and trumpeter blending bebop fire with late-night lyricism."),
  ).map(a => (
    stack(spacing: 9pt, text(size: 20pt, weight: "bold")[#a.at(0)], text(size: 10pt, weight: "bold")[#a.at(1)]),
    [#a.at(2)],
  )).flatten(),
)
