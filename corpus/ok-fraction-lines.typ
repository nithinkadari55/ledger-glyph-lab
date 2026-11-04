// expect-none
#set page(paper: "a6", margin: 1.5cm)
#set text(size: 10.5pt)
#show math.equation: set text(font: "STIX Two Math", size: 12pt)
#let mixed(w, a, b) = $#[#w] thin frac(#[#a], #[#b])$
#set par(leading: 1.2em)
Answers:

*11.* #mixed(2, 1, 6) \
*12.* #h(1.6pt)#mixed(1, 3, 4)
