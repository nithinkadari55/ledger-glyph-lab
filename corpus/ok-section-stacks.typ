// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 9pt)
#let section(title, n) = [#text(weight: 700)[#title] #list(..range(n).map(i => [Point number #(i + 1) of this part of the posting.]))]
#grid(columns: (1fr, 1fr), column-gutter: 16pt,
  stack(spacing: 10pt, section("Responsibilities", 4), section("Benefits", 4)),
  stack(spacing: 10pt, section("Requirements", 6), section("Nice to have", 4), section("Salary", 3)),
)
