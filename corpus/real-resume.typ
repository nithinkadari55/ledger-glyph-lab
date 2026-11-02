// expect-none
#set page(paper: "a4", margin: 1.6cm)
#set text(size: 10pt, lang: "en")
#show heading.where(level: 2): it => block(above: 1.2em, below: 0.6em)[
  #text(11pt, weight: "bold", fill: rgb("#24405f"), upper(it.body))
  #v(-0.4em)
  #line(length: 100%, stroke: 0.5pt + rgb("#24405f"))
]

#block(below: 1.2em)[#text(22pt, weight: "bold")[Jordan Example]]
Data Engineer · Riverton · jordan\@example.com

#grid(
  columns: (1fr, 2.6fr),
  column-gutter: 1.4em,
  [
    == Skills
    - Rust, Python, SQL
    - Stream processing
    - Data modelling
    - Cloud infrastructure

    == Languages
    English (native) \
    German (fluent)
  ],
  [
    == Experience
    *Senior Data Engineer*, Example Analytics #h(1fr) 2023–present \
    Built a streaming pipeline that processes two billion events a day and cut
    reporting latency from hours to minutes.

    *Data Engineer*, Harbour Logistics #h(1fr) 2020–2023 \
    Designed the warehouse schema for shipment tracking and introduced data
    quality checks that reduced incident tickets by a third.

    == Education
    *MSc Computer Science*, University of Riverton #h(1fr) 2020
  ],
)
