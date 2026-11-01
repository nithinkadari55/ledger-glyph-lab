// expect-none
#set page(paper: "a4", margin: 2cm)
#set text(lang: "en")
#set par(justify: true)

= Quarterly report

This paragraph contains ordinary body text that wraps across several lines without any
layout problems. It exists to make sure a clean document produces no diagnostics.

- First point
- Second point

#table(
  columns: (auto, 1fr, 1fr),
  [Item], [Q1], [Q2],
  [Revenue], [10], [12],
  [Cost], [7], [8],
)

#link("https://typst.app")[A hyperlink]

$ integral_0^1 x^2 dif x = 1/3 $
