# ledger-glyph-lab

Compiles Typst documents, checks the finished page geometry against layout
quality rules, and reports every defect as one JSON line with its exact source
location, the measured value, the limit, and how to fix it. Mechanical defects
come with machine-applicable edits. An automated author can repair its source
from this output alone, without looking at rendered pages.

## How it works

- `vendor/typst` is Typst at commit `84cca406aafe7f830ed3f37f1cc6f0c8b6b65a03`
  (Apache-2.0, see `vendor/typst/LICENSE` and `NOTICE`) with a small additive
  patch: when `Library::probes` is enabled, layout records invisible probes for
  page body, header, footer, background and foreground areas, for every box,
  block and table cell, and for every committed paragraph line (measure,
  natural width, overflow, justification, hyphenation, baseline). Exporters
  ignore probes and they are off by default, so Typst's own test suite runs
  unchanged in CI.
- The patch also adds an opt-in baseline grid: `#set page(baseline-grid: 14pt)`
  snaps the lines of top-level paragraphs to a common grid across columns and
  pages.
- `crates/lgl` compiles a document with probes on, extracts the geometry and
  runs the rules.

## Usage

```sh
lgl check doc.typ --pdf doc.pdf --geometry doc.json   # report, PDF only if no errors
lgl fix doc.typ --out doc.fixed.typ                   # apply mechanical fixes, then report
lgl rules                                             # rule catalog as JSON
```

Exit code: `0` no errors, `1` errors found, `2` invalid invocation.

Each stdout line is one diagnostic:

```json
{"v":1,"rule":"L002.container-overflow","severity":"error",
 "message":"content extends 140.2pt beyond the bottom edge of its block (481.9pt x 56.7pt) on page 1",
 "loc":{"file":"doc.typ","line":4,"col":4,"end_line":4,"end_col":14},
 "elem":"block","page":1,"bbox":{"x0":56.8,"y0":243.3,"x1":244.8,"y1":253.5},
 "measured":140.2,"limit":0.5,"unit":"pt",
 "hints":["give the container more height (auto), reduce the content, or enable clip if cutting it off is intended"],
 "fix":{"description":"let the height follow the content",
        "edits":[{"loc":{"file":"doc.typ","line":3,"col":16,"end_line":3,"end_col":19},"replacement":"auto"}]}}
```

Typst's own errors and warnings are reported as `typst.compile.*` and
`typst.pdf.*` with the same fields.

## Rules

Rules check the result, not the design: thresholds are either invariants or
derived from the document's own dominant values, so documents can look very
different and still pass. `lgl rules` prints the full catalog.

| Rule | Checks | Autofix |
|---|---|---|
| L001 page-overflow | content beyond the page edge | yes |
| L002 container-overflow | content beyond a cell or a sized/decorated box or block | yes |
| L003 margin-intrusion | body content in the margin | yes |
| L004 line-overflow | lines wider than their measure | |
| L005 loose-line | letter-spaced justified lines | |
| L006 overlap | text/text, image/text, image/image overlap | |
| L007 page-underfill | large empty area at a page bottom, with its cause | yes |
| L008 missing-glyph | characters no font can display | |
| L009 image-resolution | raster images below 150 dpi | |
| L010 orphan, L011 widow | single paragraph lines at page breaks | |
| L012 hyphen-ladder, L013 hyphen-at-page-break | hyphenation quality | |
| L014 stranded-heading | heading at a page end or followed by one line | |
| L015 inconsistent-spacing | gaps that differ from the document's own norm | yes |
| L016 figure-far-from-reference | figure more than a page from its first reference | |
| L017 misaligned-numbers | numeric table columns not aligned | |
| L018 marginal-overflow | header/footer content larger than its area | |
| L019 column-imbalance | unbalanced columns | yes |
| L020 line-length | lines too long or short to read | |
| L021 empty-page, L025 near-empty-last-page | wasted pages | |
| L022 text-too-small | text below 6pt outside math | yes |
| L023 distorted-image | stretched images | yes |
| L024 runt | last line holding a fragment | |
| L026 split-figure | figure split across pages | |
| L027 inconsistent-text-area | margins changing between pages | |
| L028 footnote-on-other-page | footnote marker and text apart | |
| L029 narrow-column | table columns too narrow to read | |
| L030 off-baseline-grid | lines off a declared baseline grid | |
| L031 uneven-line-spacing | one line pushed down by tall inline content | |
| L032 column-overflow | content running into the gutter or next column | |
| L033 leading-ratio | line pitch too tight or loose for the font size | |
| L034 top-gap | a page starting much deeper than the others | |
| L035 row-baselines | single-line cells of a row on different baselines | |
| L036 equation-number-gap | equation number touching its equation | |
| L037 wrapped-code | code lines that wrap | |
| L038 cramped-lines | lines whose ink nearly touches the line below | |
| L039 hollow-box | framed boxes much taller than their content, empty continuations | |
| L040 touching-cells | text in neighbouring cells that touches | |
| L041 mixed-precision | numbers in a column with different decimal places | |
| L042 hyphenated-display-text | hyphenated titles, headings, pull quotes | |
| L043 empty-column | pages of a multi-column layout with whole columns empty | |
| L044 centered-paragraph | wrapped running text centred across a wide measure | |
| L045 figure-too-close | figures set off from text by barely more than the line spacing | |
| L046 lopsided-columns | tables where one column wraps heavily beside a mostly empty one | |
| L047 low-contrast | text barely distinguishable from the fill or page beneath it | |
| L048 ragged-column | table text columns with neither edge aligned (centred text of different lengths) | |
| L049 broken-address | web or email addresses or phone numbers split across lines | |
| L050 line-through-text | rules, ticks or gridlines running through text | |
| L051 hyphenated-label | short labels (chart labels, narrow cells) that break a word | |
| L052 lonely-title-word | titles whose last line holds a single word | |
| L053 markup-in-text | HTML entities or Markdown markers printed literally | |
| L054 title-too-close | text under a multi-line title less than 1.5 times the title's line gap below it | |
| L055 paragraph-spacing | paragraphs barely further apart than their lines | |
| L056 unseparated-terms | wrapped term-list entries set as close as their lines | |
| L057 sparse-bar-chart | bar charts whose bar groups stand far apart | |
| L058 malformed-number | numbers with wrong digit grouping, such as 93,3.88 | |
| L059 tight-cell-padding | table text nearly touching row rules | |
| L060 double-rule | doubled separators: two rules with nothing between | |
| L061 tight-box-padding | text crowding the top or bottom of a filled box | |
| L062 dangling-separator | separator rules at the top or bottom of a column | |
| L063 chart-labels | bar chart labels detached from the axis or not centred under their bars | |
| L064 detached-subtitle | subtitle more than 0.9 title sizes below its title and no closer to it than to what follows | |
| L065 detached-underline | underline far below its display text and closer to the next text | |
| L066 floating-arrow | arrow covering less than 35% of the gap between the boxes it links | |
| L067 print-edge | content within 5 mm of the page edge that does not run off it | |
| L068 drop-cap-hole | drop cap more than one line taller than the text beside it | |
| L069 uneven-boxes | boxes side by side in a row with heights more than 10% apart | |
| L070 pair-spacing | repeated pairs spaced wider inside each pair than between pairs | |
| L071 detached-label | label more than five times its size above the document title | |
| L072 text-on-rule | text sitting on a line that runs on both sides of it | |
| L073 crowded-subtitle | subtitle less than 0.2 title sizes below a one-line title | |
| L074 detached-figure-caption | caption under a key figure further than 2.5 or closer than 0.3 of its sizes | |
| L075 off-centre-panel | one-page framed panel with uneven space above and below | |
| L076 wrong-page-label | "Page N of M" label that does not match the page or the page count | |
| L077 spread-banner | title banner whose title and subtitle are pushed to its edges | |
| L078 near-empty-page | inner page using less than a quarter of its text area | |
| S001 negative-spacing | source: large negative h()/v() | |
| S002 absolute-placement | source: place() without float, move() | |
| S003 transform-without-reflow | source: rotate/scale/skew without reflow | yes |
| S004 disabled-protection | source: orphan/widow costs or sticky headings disabled | yes |

Intended overlays such as watermarks are exempt from L006 when labelled
`<allow-overlap>`.

## Baseline grid

`#set page(baseline-grid: 15pt)` snaps every paragraph line to a grid that
starts at the top of the text area of each page. The grid phase is carried
through the layout: below floats, into blocks, quotes, figures, padding, lists,
stacks and table cells (with their insets), and across page breaks. Fractional spacing (`v(1fr)`) and
vertical alignment are rounded to whole grid steps. Choose a pitch at least as
large as the natural line pitch, or lines skip grid lines (L033 reports this).

## Known limits

- Lines inside inline boxes, footnotes and rowspanning table cells are not
  snapped to the baseline grid; text after them returns to the grid.
- Element attribution relies on Typst's introspection tags; plain inline text
  without a paragraph is treated as running text.

## License

Code in `crates/` is dual-licensed under MIT or Apache-2.0.

The fonts in `fonts/` are licensed under the SIL Open Font License 1.1, except
Special Elite (Apache License 2.0); see `fonts/README.md` and the licence file
of each family.
