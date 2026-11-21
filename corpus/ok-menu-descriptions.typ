// expect-none
#set page(paper: "a5", margin: 1.5cm)
#set text(size: 9.5pt)
#let dish(name, desc, price) = grid(columns: (1fr, auto), [*#name* \ #text(size: 8.5pt, style: "italic")[#desc]], price)
#let head(t) = block(width: 100%, stroke: (bottom: 0.6pt), inset: (bottom: 3pt), above: 24pt, below: 8pt)[#text(size: 15pt, weight: 700)[#t]]
#align(center, text(size: 30pt, weight: 700)[Tsuki no Ie])
#head[Rice & Noodles]
#stack(spacing: 8pt,
  dish("Yaki udon", "Stir-fried udon, cabbage, pork belly, bonito", "13.00"),
  dish("Garlic fried rice", "Egg, spring onion, shoyu butter", "9.00"),
)
#head[Desserts]
#stack(spacing: 8pt,
  dish("Matcha tiramisu", "Mascarpone, sponge soaked in green tea", "7.00"),
  dish("Mochi trio", "Strawberry, mango and red bean", "6.00"),
)
