// expect: L045.figure-too-close
#set page(paper: "a4", margin: 2cm)
#set text(size: 10pt)
#show figure: set block(above: 0.6em, below: 0.6em)
#show figure.where(kind: table): set figure.caption(position: top)
The reference value was computed to machine precision with Newton's method.
Table 1 lists the objective gap after ten, one hundred and one thousand
iterations next to the guarantee of the theorem.
#figure(
  table(columns: 2, [Iteration], [Gap], [10], [0.31], [100], [0.022]),
  caption: [Objective gap of gradient descent against the bound.],
)
In every row the observed gap lies below the bound, as the theorem requires,
and the ratio between bound and observation stays near one order of magnitude.
