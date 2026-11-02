// expect-none
#show figure: set block(above: 2em, below: 2em)
#set page(paper: "us-letter", margin: 1in, numbering: "1")
#set text(size: 11pt, lang: "en")
#set par(justify: true)
#set heading(numbering: "1.")
#set math.equation(numbering: "(1)")

#align(center)[
  #text(16pt, weight: "bold")[On the Convergence of Damped Iterations] \
  #v(0.3em)
  A. Author · Department of Mathematics
]

#v(1em)

*Abstract.* We study fixed-point iterations of the form $x_(k+1) = (1 - alpha) x_k + alpha T(x_k)$
for a nonexpansive operator $T$ and show linear convergence under a mild
regularity assumption. Numerical experiments confirm the predicted rates.

= Introduction

Fixed-point methods are central to optimisation and numerical analysis. Given a
map $T: RR^n -> RR^n$, the damped iteration in @eq:iter averages the current
point with its image under $T$.

$ x_(k+1) = (1 - alpha) x_k + alpha T(x_k), quad 0 < alpha < 1 $ <eq:iter>

If $T$ is a contraction with constant $q < 1$, then the error satisfies

$ norm(x_(k+1) - x^*) <= (1 - alpha (1 - q)) norm(x_k - x^*), $ <eq:rate>

so the iteration converges linearly. @eq:rate also shows that the optimal step
is $alpha = 1$ in this case.

= Main result

#let theorem(body) = block(
  inset: (left: 1em),
  stroke: (left: 1pt + luma(150)),
)[*Theorem.* #body]

#theorem[
  Let $T$ be nonexpansive and metrically subregular at $x^*$ with modulus
  $kappa > 0$. Then for every $alpha in (0, 1)$ the damped iteration converges
  linearly, with rate $sqrt(1 - alpha (1 - alpha) kappa^2)$.
]

The proof combines the averaged-operator identity with the regularity bound and
is given in the appendix. The rate degrades gracefully as $kappa -> 0$, which
matches the sublinear behaviour known for general nonexpansive maps.

= Experiments

We test the method on a family of random linear problems. @tab:results reports
the observed rates, which agree with the bound to two decimal places.

#figure(
  table(
    columns: 4,
    align: (center, right, right, right),
    table.header([$n$], [$kappa$], [predicted], [observed]),
    [100], [0.30], [0.977], [0.975],
    [200], [0.25], [0.984], [0.983],
    [400], [0.20], [0.990], [0.990],
  ),
  caption: [Predicted and observed linear rates.],
) <tab:results>

= Conclusion

Metric subregularity suffices for linear convergence of damped iterations, and
the rate is explicit. Extensions to stochastic operators are left for future
work.
