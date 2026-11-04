// expect-none
#set page(paper: "a6", margin: 1.5cm)
#let f(a, b) = $frac(#[#a], #[#b])$
#let fr(a, b) = $frac(#[#a], #[#b])$
Add: $#f(1, 4) + fr(2, 4)$.

#let g = 2
A function: $h(x) = x^2 + g$.
