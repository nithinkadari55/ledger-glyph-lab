// expect: L044.centered-paragraph
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#block(width: 100%, inset: 8pt, fill: luma(240))[
  #align(center)[
    *Theorem 1.* _Let f be convex and smooth with a Lipschitz continuous gradient,
    and let the iterates be generated with a constant step size. Then for every k._
  ]
]
