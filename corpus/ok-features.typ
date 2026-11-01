// expect-none
#set document(title: "Feature coverage")
#set page(
  paper: "a4",
  margin: (x: 2.2cm, y: 2.5cm),
  header: [Feature coverage #h(1fr) Internal],
  footer: context [#h(1fr) Page #counter(page).display() #h(1fr)],
)
#set text(lang: "en", size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.1")
#set math.equation(numbering: "(1)")
#show link: underline

#outline()

= Introduction <intro>

Ångström, Évian and Øresund carry accents above the cap height. Inline math such as
$a^2 + b^2 = c^2$ and $sum_(i=1)^n i = n(n+1)/2$ sits on the baseline, and a footnote
follows here.#footnote[Footnotes are placed at the bottom of the page.] See
@results for the results and #link("https://typst.app/docs")[the documentation].

#lorem(80)

== Lists

- A bullet item with some words in it.
- A second item that is long enough to wrap onto a second line of the list body text.
  - A nested item.
+ Numbered one.
+ Numbered two.

/ Term: Its description, which also wraps if it gets long enough to need it.

= Results <results>

#figure(
  table(
    columns: (auto, 1fr, 1fr, 1fr),
    align: (left, right, right, right),
    table.header([*Region*], [*Q1*], [*Q2*], [*Q3*]),
    [North], [1,204], [1,388], [1,502],
    [South], [988], [1,021], [1,130],
    table.cell(colspan: 2)[Merged total], [2,409], [2,632],
  ),
  caption: [Quarterly figures by region.],
)

#figure(
  image("assets/chart.svg", width: 70%),
  caption: [A bar chart.],
)

$ integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2 $ <gauss>

As shown in @gauss, the integral converges.

```rust
fn main() {
    println!("short line of code");
}
```

#set par(justify: false)

#grid(
  columns: (1fr, 1fr),
  gutter: 12pt,
  rect(width: 100%, inset: 8pt)[#lorem(20)],
  rect(width: 100%, inset: 8pt)[#lorem(25)],
)

#align(center)[#rotate(90deg, reflow: true)[Rotated]]

#columns(2)[
  #lorem(120)
]

#quote(block: true, attribution: [Someone])[#lorem(30)]

#place(bottom + right, float: true)[#box(fill: luma(230), inset: 6pt)[Floating note]]

#lorem(150)
