// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 10pt)
// Month grid: day numbers left in their cells.
#grid(columns: (1fr,) * 7, rows: 28pt, stroke: 0.4pt,
  ..range(1, 29).map(d => pad(3pt, text(weight: 700)[#d])))
#v(1em)
// Answer key: numbered labels and fractions set as math.
#grid(columns: (auto, 1fr, auto, 1fr), column-gutter: 6pt, row-gutter: 1.2em,
  [*1.*], $1/6$, [*2.*], $2/15$,
  [*3.*], $3/8$, [*4.*], $6/20 = 3/10$,
  [*5.*], $4$, [*6.*], $9$,
  [*7.*], $1/2$, [*8.*], $7/12$,
  [*9.*], $5/9$, [*10.*], $2/3$,
  [*11.*], $1/4$, [*12.*], $3/14$,
)
#v(1em)
// Poster acts: large ordinals.
#grid(columns: 1, row-gutter: 14pt,
  ..("01", "12", "31").map(n => text(size: 30pt, weight: 900)[#n]))
