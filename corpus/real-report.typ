// expect-none
#set document(title: "Quarterly Operations Report")
#set page(
  paper: "a4",
  margin: (x: 2.2cm, top: 2.6cm, bottom: 2.4cm),
  header: context {
    if counter(page).get().first() > 1 [
      #set text(9pt, fill: luma(90))
      Quarterly Operations Report #h(1fr) Q3 2026
    ]
  },
  footer: context [
    #set text(9pt, fill: luma(90))
    #h(1fr) #counter(page).display("1 / 1", both: true)
  ],
)
#set text(font: "Libertinus Serif", size: 10.5pt, lang: "en")
#set par(justify: true, leading: 0.62em)
#set heading(numbering: "1.1")
#show heading.where(level: 1): set block(above: 1.6em, below: 0.9em)
#show link: set text(fill: rgb("#1f4e8c"))

#align(center)[
  #text(20pt, weight: "bold")[Quarterly Operations Report]
  #v(0.4em)
  #text(12pt)[Third quarter 2026 · Prepared by the Operations Office]
]

#v(1em)

= Summary

Operations improved across all regions this quarter. Fulfilment time fell by
twelve percent, and customer satisfaction rose to its highest level in two
years. The sections below detail the drivers, the regional results and the
priorities for the next quarter. Key figures are collected in @kpis.

= Regional results

#figure(
  table(
    columns: (1fr, auto, auto, auto),
    align: (left, right, right, right),
    inset: (x: 8pt, y: 5pt),
    stroke: (x, y) => if y == 0 { (bottom: 0.7pt) } else { none },
    table.header([*Region*], [*Orders*], [*Revenue (k)*], [*Margin*]),
    [North], [12,480], [1,204.5], [18.2%],
    [South], [9,915], [988.0], [16.9%],
    [East], [11,230], [1,130.7], [17.4%],
    [West], [8,402], [842.3], [15.1%],
  ),
  caption: [Orders, revenue and margin by region.],
) <kpis>

The northern region again led on volume, while the eastern region showed the
strongest growth. Margins remained stable despite higher freight costs.

== Drivers

- *Warehouse automation* reduced picking errors and shortened order cycles.
- *Carrier consolidation* lowered average shipping cost per parcel.
- *Demand forecasting* cut stock-outs on the fifty fastest-moving products.

== Risks

+ Freight rates may rise again in the holiday season.
+ Two suppliers remain single points of failure for packaging.

= Next quarter

The priorities are to extend automation to the southern warehouse, renegotiate
packaging contracts, and pilot same-day delivery in two cities. Progress will be
tracked in the monthly operations review.#footnote[Reviews are held on the first
Tuesday of each month.] Further details are available on the
#link("https://example.com/ops")[operations portal].
