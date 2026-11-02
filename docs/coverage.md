# Coverage of the layout quality rules

The audit behind this project listed 65 layout quality rules (H01-H65). This
table maps each one to the check that covers it, or gives the reason it is not
checked.

| Rule | Topic | Covered by |
|---|---|---|
| H01 | ink within the page | L001 |
| H02 | ink within the content area | L003 |
| H03 | content within its column | L032 |
| H04 | cell content within its cell | L002 |
| H05 | fixed-size box/block content within it | L002 |
| H06 | no overlapping elements | L006 |
| H07 | header/footer within their areas | L018 |
| H08 | no unintended clipping | not checked: clipping is always declared explicitly (`clip: true`, `fit: "cover"`) |
| H09 | footnote marker and entry on one page | L028 |
| H10 | margin notes do not collide | L006 (text overlap) |
| H11 | consistent gaps per role pair | L015 |
| H12 | no stacked spacing | L015 (outlier gaps) |
| H13 | constant baseline distance in a paragraph | L031 |
| H14 | leading ratio | L033 |
| H15 | no extra space at the top of a page | L034 |
| H16 | page fill | L007 |
| H17 | no unintended internal gap | partly L015; fractional spacing is deliberate and leaves no trace |
| H18 | no empty pages | L021 |
| H19 | last page not near-empty | L025 |
| H20 | balanced columns | L019 |
| H21 | no blank line boxes | L015 (resulting gaps) |
| H22 | no overfull lines | L004 |
| H23 | justification ratio within bounds | L005 |
| H24 | word-space bound | L005 (stretch exhausted) |
| H25 | rivers | not checked: close to a patented method (US11960818B2) |
| H26 | no runt last line | L024 |
| H27 | last line never stretched | enforced by Typst |
| H28 | consecutive hyphens limited | L012 |
| H29 | no hyphen across a page break | L013 |
| H30 | hyphenation fragment length | enforced by the hyphenation patterns |
| H31 | line length | L020 |
| H32 | no widows or orphans | L010, L011 |
| H33 | heading not last on a page | L014 |
| H34 | heading followed by two lines | L014 |
| H35 | caption with its figure | L026 |
| H36 | float near its first reference | L016 |
| H37 | floats in numbering order | enforced by Typst's float queue |
| H38 | figure/image within the scope width | L001, L003, L032 |
| H39 | table within the available width | L001, L002, L003 |
| H40 | column at least min-content wide | L002, L004 (resulting overflow) |
| H41 | no over-narrow wrapped columns | L029 |
| H42 | table header repeats | enforced by Typst for `table.header` |
| H43 | no header-only table fragment | enforced by Typst's orphan-header prevention |
| H44 | numeric columns aligned | L017 |
| H45 | row baselines aligned | L035 |
| H46 | left edges of same-role blocks align | not checked: indents, lists and quotes make roles ambiguous without author intent |
| H47 | centred/end-aligned items symmetric | violated only through overflow: L001-L003, L032 |
| H48 | baselines align across columns | baseline grid (page.baseline-grid) + L030 |
| H49 | inline objects on the baseline | not checked: depends on the author's declared box baseline |
| H50 | image aspect ratio preserved | L023 |
| H51 | image resolution floor | L009 |
| H52 | upscaling limit | L009 |
| H53 | display equation fits; number apart | L003/L032, L036 |
| H54 | inline math within the line | L004 |
| H55 | code does not overflow; wrapping reported | L004, L037 |
| H56 | link rectangles cover the linked ink | not checked: rectangles are produced by Typst from the linked content |
| H57 | internal links resolve | Typst errors on missing labels |
| H58 | consistent font size per role | not checked: would restrict deliberate design (titles, emphasis) |
| H59 | minimum readable size | L022 |
| H60 | no missing glyphs | L008 |
| H61 | no unintended font fallback | Typst warns about unknown font families; fallback for missing coverage is reported by L008 |
| H62 | consistent margins | L027 |
| H63 | consistent header/footer positions | not checked: first pages and section openers legitimately differ |
| H64 | document converged | Typst warning, reported as `typst.compile.warning` |
| H65 | no dropped content | Typst warning, reported as `typst.compile.warning` |
