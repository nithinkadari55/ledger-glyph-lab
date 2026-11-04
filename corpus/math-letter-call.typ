// expect: S005.math-letter-call
#set page(paper: "a6", margin: 1.5cm)
#let f(a, b) = $frac(#[#a], #[#b])$
Add: $f(1, 4) + f(2, 4)$.
