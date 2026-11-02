// expect-none
#show figure: set block(above: 2em, below: 2em)
#import "@preview/cetz:0.4.2"
#set page(paper: "a5", margin: 2cm)
#set text(lang: "en")

= Process overview

The pipeline has three stages, shown below.

#figure(
  cetz.canvas({
    import cetz.draw: *
    rect((0, 0), (2, 1), name: "a")
    content("a", [Ingest])
    rect((3, 0), (5, 1), name: "b")
    content("b", [Transform])
    rect((6, 0), (8, 1), name: "c")
    content("c", [Publish])
    line("a.east", "b.west", mark: (end: ">"))
    line("b.east", "c.west", mark: (end: ">"))
  }),
  caption: [The three pipeline stages.],
)

Each stage runs independently and communicates through durable queues.
