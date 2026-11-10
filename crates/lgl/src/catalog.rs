use serde::Serialize;

/// A description of one rule, for agents and documentation.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct RuleInfo {
    pub id: &'static str,
    pub severity: &'static str,
    pub checks: &'static str,
    pub threshold: &'static str,
    pub repair: &'static str,
    pub autofix: bool,
}

const fn rule(
    id: &'static str,
    severity: &'static str,
    checks: &'static str,
    threshold: &'static str,
    repair: &'static str,
    autofix: bool,
) -> RuleInfo {
    RuleInfo { id, severity, checks, threshold, repair, autofix }
}

pub const CATALOG: &[RuleInfo] = &[
    rule(
        "L001.page-overflow",
        "error",
        "Visible content extends beyond the page edge (plus declared bleed).",
        "0.5pt",
        "Use relative (%) or fractional (fr) sizes, let content wrap or break.",
        true,
    ),
    rule(
        "L002.container-overflow",
        "error",
        "Content extends beyond a cell, an explicitly sized box/block, or a box/block \
         with fill or stroke, without clip.",
        "0.5pt (text: 0.35em, but 0.15em for a letter or digit at the right end)",
        "Give the container more room (auto, fr, %), let content wrap, or clip on \
         purpose.",
        true,
    ),
    rule(
        "L003.margin-intrusion",
        "error",
        "Body content extends into the page margin.",
        "0.5pt (text: 0.35em, but 0.15em for a letter or digit at the right end)",
        "Keep content within the text area: relative sizes, smaller content.",
        true,
    ),
    rule(
        "L004.line-overflow",
        "error",
        "A paragraph line is wider than its measure (long word, URL, wide inline box).",
        "0.5pt",
        "Allow breaks (hyphenation, #sym.zws, soft hyphen) or shorten the content.",
        false,
    ),
    rule(
        "L005.loose-line",
        "warning",
        "A justified line ran out of word-space stretch and was letter-spaced.",
        "0.5pt extra per character",
        "Enable hyphenation, rephrase, or widen the column.",
        false,
    ),
    rule(
        "L006.overlap",
        "error",
        "Text overlaps text, or an image overlaps text or another image.",
        "0.25em for text pairs, 2pt with images",
        "Keep content in the normal flow (no place/move/negative spacing). Label \
         intended overlays <allow-overlap>.",
        false,
    ),
    rule(
        "L007.page-underfill",
        "warning",
        "A page that ran out of space leaves a large empty area at its bottom.",
        "20% of the text area",
        "Let the following content break (breakable: true) or float, or reduce it.",
        true,
    ),
    rule(
        "L008.missing-glyph",
        "error",
        "No available font can display a character; it renders as an empty box.",
        "any",
        "Add a fallback font covering the characters, or replace them.",
        false,
    ),
    rule(
        "L009.image-resolution",
        "warning",
        "A raster image is shown below print resolution.",
        "150 dpi",
        "Show the image smaller or use a higher-resolution source.",
        false,
    ),
    rule(
        "L010.orphan",
        "warning",
        "The first line of a paragraph is alone at the bottom of a page.",
        "1 line",
        "Keep default widow/orphan costs; adjust preceding content.",
        false,
    ),
    rule(
        "L011.widow",
        "warning",
        "The last line of a paragraph is alone at the top of a page.",
        "1 line",
        "Keep default widow/orphan costs; adjust paragraph length.",
        false,
    ),
    rule(
        "L012.hyphen-ladder",
        "warning",
        "Too many consecutive lines end with an inserted hyphen.",
        "2 lines",
        "Rephrase or adjust the text width.",
        false,
    ),
    rule(
        "L013.hyphen-at-page-break",
        "warning",
        "A hyphenated word is split across a page break.",
        "any",
        "Rephrase the line or adjust preceding content.",
        false,
    ),
    rule(
        "L014.stranded-heading",
        "warning",
        "A heading ends a page, or is followed by only one line of its section.",
        "fewer than 2 following lines",
        "Keep headings sticky (default); adjust preceding content.",
        false,
    ),
    rule(
        "L015.inconsistent-spacing",
        "warning",
        "A gap between blocks differs from the document's own dominant gap for the \
         same pair of roles.",
        "max(1.5pt, 20% of the dominant gap)",
        "Remove manual v()/linebreaks/empty paragraphs; set spacing once with set rules.",
        true,
    ),
    rule(
        "L016.figure-far-from-reference",
        "warning",
        "A figure is placed more than one page away from its first reference.",
        "1 page",
        "Move the figure next to the paragraph that first references it.",
        false,
    ),
    rule(
        "L017.misaligned-numbers",
        "warning",
        "Numbers in a table column do not line up on their digits.",
        "1pt spread of right edges",
        "Right-align numeric columns and use consistent decimals.",
        false,
    ),
    rule(
        "L018.marginal-overflow",
        "error",
        "Header or footer content does not fit its area.",
        "0.5pt",
        "Shorten the header/footer, reduce its size, or enlarge its margin.",
        false,
    ),
    rule(
        "L019.column-imbalance",
        "warning",
        "The last column ends far above the others: by more than 3 lines on a \
         non-final page, or by more than a third of the column height on the last page.",
        "3 lines; last page 1/3 of the column height",
        "Balance the columns (columns(.., balanced: true)).",
        true,
    ),
    rule(
        "L020.line-length",
        "warning",
        "Justified body lines are too long or too short to read comfortably.",
        "30-115 characters (median)",
        "Change font size, margins or column count.",
        false,
    ),
    rule(
        "L021.empty-page",
        "warning",
        "A page has no content in its text area.",
        "any",
        "Remove consecutive page breaks.",
        false,
    ),
    rule(
        "L022.text-too-small",
        "warning",
        "Text outside math is set below a readable size.",
        "6pt",
        "Use at least 6pt.",
        true,
    ),
    rule(
        "L023.distorted-image",
        "warning",
        "An image is shown with a different aspect ratio than its source.",
        "1%",
        "Set only width or height, or use fit: \"contain\"/\"cover\".",
        true,
    ),
    rule(
        "L024.runt",
        "warning",
        "The last line of a paragraph holds only a word fragment.",
        "8% of the measure",
        "Rephrase slightly.",
        false,
    ),
    rule(
        "L025.near-empty-last-page",
        "warning",
        "Content spills onto a last page that it barely fills.",
        "10% of the text area",
        "Tighten preceding content or add closing content.",
        false,
    ),
    rule(
        "L026.split-figure",
        "warning",
        "A figure is split across pages.",
        "any",
        "Reduce the figure, or let long tables break outside a figure.",
        false,
    ),
    rule(
        "L027.inconsistent-text-area",
        "warning",
        "The text area differs between pages of the same size.",
        "0.5pt (mirrored margins allowed)",
        "Set margins once at the start.",
        false,
    ),
    rule(
        "L028.footnote-on-other-page",
        "warning",
        "A footnote marker and its text are on different pages.",
        "any",
        "Shorten the footnote or move it earlier.",
        false,
    ),
    rule(
        "L029.narrow-column",
        "warning",
        "A table column is so narrow that its text wraps into many lines.",
        "60pt wide and 4 lines",
        "Give the column more room (auto or a larger fr share).",
        false,
    ),
    rule(
        "L030.off-baseline-grid",
        "error",
        "With page(baseline-grid: ..), a top-level paragraph line is off the grid.",
        "0.05pt",
        "Avoid fractional spacing and bottom alignment around grid text.",
        false,
    ),
    rule(
        "L031.uneven-line-spacing",
        "warning",
        "A line in a paragraph sits much further below the previous one than usual, \
         typically because of tall inline content.",
        "25% above the paragraph's median pitch, plus 1pt",
        "Move tall inline content into a display block, or reduce it.",
        false,
    ),
    rule(
        "L032.column-overflow",
        "error",
        "Content extends beyond its column into the gutter or the next column.",
        "0.5pt (text: 0.35em, but 0.15em for a letter or digit at the right end)",
        "Use relative or fractional widths inside columns, or span columns with \
         place(scope: \"parent\", float: true).",
        false,
    ),
    rule(
        "L033.leading-ratio",
        "warning",
        "Running text lines are too close or too far apart for their font size.",
        "line pitch / font size within 1.1-1.8 (median)",
        "Adjust par(leading: ..).",
        false,
    ),
    rule(
        "L034.top-gap",
        "warning",
        "Content on a page starts much deeper than on the other pages.",
        "12pt deeper than the median page (first page exempt)",
        "Remove manual vertical space at the start of the page.",
        false,
    ),
    rule(
        "L035.row-baselines",
        "warning",
        "Single-line cells of a table row have different baselines.",
        "0.5pt",
        "Use the same text size and vertical alignment within a row.",
        false,
    ),
    rule(
        "L036.equation-number-gap",
        "warning",
        "A display equation's number touches or overlaps the equation.",
        "0.5em (error when overlapping)",
        "Split the equation over several lines or shorten it.",
        false,
    ),
    rule(
        "L037.wrapped-code",
        "warning",
        "Lines of a code block are too long and wrap.",
        "any wrapped line",
        "Shorten the lines or use a smaller code size.",
        false,
    ),
    rule(
        "L038.cramped-lines",
        "error/warning",
        "Two stacked lines nearly touch: their baseline distance leaves too little room \
         for a typical 0.25em descent and 0.75em ascent (too little leading for large \
         text, or too little space around a title). Error when they collide, warning \
         when tight.",
        "clear space of 0.1em of the larger text size",
        "Give large text more leading in its own size, or more space between blocks.",
        false,
    ),
    rule(
        "L039.hollow-box",
        "error/warning",
        "A box or block with fill or stroke is much taller than its content, usually \
         from height: 100% or 1fr, leaving a large empty frame (or an empty continuation \
         on the next page). Error when the empty part exceeds a quarter page or the box \
         is empty. Label intended empty boxes <allow-empty>.",
        "72pt empty and content under 40% of the height",
        "Remove the fixed height so the box fits its content; size the grid row instead.",
        false,
    ),
    rule(
        "L040.touching-cells",
        "error",
        "Text in neighbouring grid or table cells on the same line touches or nearly \
         touches, so label and value read as one word.",
        "0.2em gap",
        "Add column-gutter or inset to the grid or table.",
        false,
    ),
    rule(
        "L041.mixed-precision",
        "warning",
        "Numbers in one table column use different numbers of decimal places (23% next \
         to 29.9%), so their decimal points do not line up.",
        "3 numeric cells, any difference",
        "Write every number in the column with the same number of decimal places.",
        false,
    ),
    rule(
        "L042.hyphenated-display-text",
        "warning",
        "A line of display text (title, heading, pull quote) ends with a hyphen.",
        "14pt text",
        "Turn hyphenation off for display text and break lines by wording or width.",
        false,
    ),
    rule(
        "L043.empty-column",
        "warning",
        "A page of a multi-column layout leaves whole columns empty, typically the last \
         page where the text runs out in the first column.",
        "any empty column",
        "Balance the last page's columns, move content there, or change the page count.",
        false,
    ),
    rule(
        "L044.centered-paragraph",
        "warning",
        "Running text below display size wraps across a wide measure and is centred, so \
         its lines start at different places.",
        "measure over 60% of the text area",
        "Align running text left or justify it; centre only single lines.",
        false,
    ),
    rule(
        "L045.figure-too-close",
        "warning",
        "A figure or table is set off from the text above or below it by less than \
         about a blank line, so it and its caption read as part of the paragraph \
         (print convention is 1.5 to 2 lines).",
        "one line pitch beyond the line spacing",
        "Give figures and tables about a blank line above and below.",
        false,
    ),
    rule(
        "L046.lopsided-columns",
        "warning",
        "A table's column widths are badly distributed: one column wraps its cells into \
         many lines while a neighbouring column stays on one line and is mostly empty.",
        "3 lines against a single line under 60% full with 40pt to spare, in 2 rows",
        "Give the wrapping column a larger share and let short columns size to content.",
        false,
    ),
    rule(
        "L047.low-contrast",
        "error/warning",
        "Text barely differs in lightness from the solid fill (or white page) beneath \
         it, such as white text on a pale header row. Error below 1.5:1, where the text \
         all but disappears. Text on images and gradients is not judged.",
        "WCAG contrast ratio 3:1",
        "Darken the text or lighten its background (or the reverse).",
        false,
    ),
    rule(
        "L048.ragged-column",
        "warning",
        "A table column of text has no straight edge: below the header, its cells \
         neither start nor end at a common position, as with centred text of different \
         lengths or numbered labels whose numbers do not line up.",
        "6pt spread of both edges, 3 text cells",
        "Align text columns left and numeric columns right.",
        false,
    ),
    rule(
        "L049.broken-address",
        "warning",
        "A web or email address wraps onto the next line without a hyphen, such as \
         \"www.example.\" followed by \"com\", or an international phone number is split \
         across lines.",
        "any split address or number",
        "Keep the address on one line: shorten the text around it or wrap it in box().",
        false,
    ),
    rule(
        "L050.line-through-text",
        "error/warning",
        "A thin line (rule, tick, gridline) runs through text, or a horizontal line \
         touches its ink, such as a heading rule set right under the descenders. \
         Underline, strike, overline, highlight and content labelled <allow-overlap> \
         are exempt.",
        "lines up to 3pt thick; through 0.5pt deep (error); 0.08em clearance (warning)",
        "End lines before labels, move the text, or give labels a background.",
        false,
    ),
    rule(
        "L051.hyphenated-label",
        "warning",
        "A short label (at most 3 lines in a measure under 120pt, such as a chart label, \
         a stat caption or a narrow cell) breaks a word with a hyphen.",
        "any hyphen",
        "Widen the label's column, shorten it, or turn hyphenation off for labels.",
        false,
    ),
    rule(
        "L052.lonely-title-word",
        "warning",
        "The last line of a title or other display text (14pt or larger) holds a single \
         word below a line of three or more.",
        "one word",
        "Balance the title's lines with an earlier line break, a narrower block or new \
         wording.",
        false,
    ),
    rule(
        "L053.markup-in-text",
        "error",
        "An HTML entity (&nbsp;, &mdash;, &#8211;) or a Markdown marker (**) is printed \
         literally, outside code.",
        "any",
        "Use Typst syntax: ~ for a non-breaking space, the character itself or sym.*, \
         *bold* and _italic_.",
        false,
    ),
    rule(
        "L054.title-too-close",
        "warning",
        "Text below a multi-line title or headline (14pt or larger) starts less than 1.5 \
         times the gap between the title's own lines below it, so it reads as part of the \
         title.",
        "1.5 times the gap between the title's lines",
        "Leave clearly more space below the title than between its lines.",
        false,
    ),
    rule(
        "L055.paragraph-spacing",
        "warning",
        "Body paragraphs without a first-line indent are set barely further apart than \
         their lines, so they run together (typically after spacing was squeezed to meet \
         a page count).",
        "0.5em more than the line pitch",
        "Set par(spacing) at least 0.5em above par(leading), or indent first lines; cut \
         words rather than spacing to meet a page count.",
        false,
    ),
    rule(
        "L056.unseparated-terms",
        "warning",
        "The entries of a term list wrap but are set as close as their own lines, so \
         without markers one entry runs into the next.",
        "0.25em more than the line pitch",
        "Set terms(tight: false) or terms(spacing: ..), or set descriptions below terms.",
        false,
    ),
    rule(
        "L057.sparse-bar-chart",
        "warning",
        "The bars of a chart (standing on an axis line) are far apart: the gap between \
         bar groups is wider than 1.5 times a group, so the chart looks empty.",
        "gap 1.5x the group width",
        "Widen the bars or narrow the chart, especially with few categories.",
        false,
    ),
    rule(
        "L058.malformed-number",
        "error",
        "A number written with both a comma and a point is grouped wrongly, such as \
         \"93,3.88\" for 933.88, typically from a faulty formatting helper.",
        "groups of exactly three digits",
        "Fix the number formatting and check the values computed with it.",
        false,
    ),
    rule(
        "L059.tight-cell-padding",
        "warning",
        "Text in a table cell keeps less than 0.15em of clear space to the row rule \
         above or below it (Typst's default inset leaves about 0.2em). Reported once \
         per table.",
        "0.15em",
        "Increase the table's vertical inset.",
        false,
    ),
    rule(
        "L060.double-rule",
        "warning",
        "Two parallel horizontal rules of the same length run 3 to 36pt apart with \
         nothing between them, so a separator is doubled (a section's closing rule and \
         the next section's opening rule or top border).",
        "any",
        "Keep one of the two rules.",
        false,
    ),
    rule(
        "L061.tight-box-padding",
        "warning",
        "Text in a filled box sits closer than 0.25em of its own size to the top or \
         bottom edge of the fill.",
        "0.25em",
        "Increase the box's inset on that side, scaled to its largest text.",
        false,
    ),
    rule(
        "L062.dangling-separator",
        "warning",
        "A separator rule stands at the very top or bottom of a column with nothing on \
         its other side, so it separates nothing (tables, figures and headings are \
         exempt).",
        "any",
        "Draw separators only between items.",
        false,
    ),
    rule(
        "L063.chart-labels",
        "warning",
        "The category labels of a bar chart sit far below the bars (more than 2.5 times \
         their size, detached from the axis), or are not centred under their bars (off by \
         more than 0.3 bar widths).",
        "2.5em gap; 0.3 bar widths offset",
        "Place labels directly under the axis, laid out in the same columns as the bars.",
        false,
    ),
    rule(
        "L064.detached-subtitle",
        "warning",
        "A subtitle, standfirst or tagline in the title area starts more than 0.9 title          sizes below its title and no closer to the title than to what follows, so it          drifts away from the title.",
        "0.9 title size",
        "Keep the subtitle close to the title; put the larger space after the title block.",
        false,
    ),
    rule(
        "L065.detached-underline",
        "warning",
        "A line under display text (a name, a headline), no wider than the text, sits more          than half the text size below it and closer to the text that follows.",
        "0.5 text size",
        "Move the line up to its text, or space it evenly as a separator.",
        false,
    ),
    rule(
        "L066.floating-arrow",
        "warning",
        "An arrow between two boxes spans less than 35% of the gap between them, so it          floats instead of connecting them.",
        "35% of the gap",
        "Move the boxes closer, or draw the arrow across the gap.",
        false,
    ),
    rule(
        "L067.print-edge",
        "warning",
        "Content comes closer than 5 mm to the page edge without running off it, so          printers may cut it off.",
        "5 mm (14pt)",
        "Keep content 5 mm inside the edge; let backgrounds run off the page.",
        false,
    ),
    rule(
        "L068.drop-cap-hole",
        "warning",
        "A drop cap is more than one line taller than the text beside it, leaving a hole.",
        "1 body size",
        "Start the drop cap on a longer paragraph, or let it span fewer lines.",
        false,
    ),
    rule(
        "L069.uneven-boxes",
        "warning",
        "Boxes side by side in a row, tops level, differ in height by more than 10%, so          the row looks ragged.",
        "0.9 height ratio",
        "Give the boxes one row height, or stack them.",
        false,
    ),
    rule(
        "L070.pair-spacing",
        "warning",
        "Three or more repeated pairs (label and value, command and description) are spaced          wider inside each pair than between pairs, so each second part reads as belonging          to the next pair.",
        "space between pairs",
        "Less space inside a pair, more space between pairs.",
        false,
    ),
    rule(
        "L071.detached-label",
        "warning",
        "A small label floats free of what it names: a kicker more than five times its own \
         size above the document title, or a short label above a larger value (a stat, a \
         price, a list) more than twice as far from the value as from whatever is above it.",
        "5 label sizes (kicker); 2 times the gap above it (label and value)",
        "Keep the label about a third of the value's size above it (stack(spacing: ..)).",
        false,
    ),
    rule(
        "L072.text-on-rule",
        "warning",
        "A text line sits on a horizontal line that runs on both sides of it, level with          its bottom: the block below has run into the text.",
        "any",
        "Leave a fixed gap between the text and the block below, or shorten the text.",
        false,
    ),
    rule(
        "L073.crowded-subtitle",
        "warning",
        "A subtitle or standfirst starts less than 0.2 title sizes below a one-line title, \
         so the two crowd each other.",
        "0.2 title size",
        "Leave about 0.3 to 0.6 title sizes between title and subtitle.",
        false,
    ),
    rule(
        "L074.detached-figure-caption",
        "warning",
        "The caption under a key figure (a short number set at least 1.8 times larger) \
         sits more than 2.5 caption sizes below it, or less than 0.3, so figure and caption \
         fall apart or crowd each other.",
        "0.3 to 2.5 caption sizes",
        "Set the caption directly under its figure.",
        false,
    ),
    rule(
        "L075.off-centre-panel",
        "warning",
        "The framed panel holding a one-page document (at least half the page high) has \
         more than 1.5 times as much space on one side (top or bottom) as on the other.",
        "1.5",
        "Centre the panel on the page, or let it fill the page height.",
        false,
    ),
    rule(
        "L076.wrong-page-label",
        "error",
        "A \"Page N of M\" label in a header or footer does not match the page it is on or \
         the document's page count.",
        "exact",
        "Number pages with counter(page), not typed numbers.",
        false,
    ),
    rule(
        "L077.spread-banner",
        "warning",
        "In a title banner holding a title and one subtitle line, the gap between them is \
         more than 2.5 times the banner's top or bottom padding, so they look pushed to the \
         edges.",
        "2.5",
        "Bring the subtitle closer to the title and give the banner more inset.",
        false,
    ),
    rule(
        "L078.near-empty-page",
        "warning",
        "A page inside the document holds less than a quarter of its text area, spilled \
         over from a full page before it, even when a page break follows, so it is almost \
         blank.",
        "25% of the text area",
        "Fit the spilled content on the previous page, or remove the page break.",
        false,
    ),
    rule(
        "L080.box-over-text",
        "error",
        "A filled or framed box is painted over part of text outside it.",
        "1pt",
        "Leave a fixed gap before the box.",
        false,
    ),
    rule(
        "L081.split-box",
        "error",
        "A filled or framed box breaks across pages leaving fewer than three lines in one          of its pieces.",
        "3 lines",
        "Keep a short box in one piece with block(breakable: false), or make room for it.",
        false,
    ),
    rule(
        "L082.clipped-text",
        "error",
        "Text runs past the edge of a box that clips its content (often a box clipped \
         only to round its corners), so part of it is not printed.",
        "0.5pt or 0.1em",
        "Give the text room inside the box: a smaller size, shorter text or a wider box.",
        false,
    ),
    rule(
        "L083.stranded-separator",
        "warning",
        "An inline list joined by separators (·, |, •) wraps and leaves a separator at \
         the end or the start of a line.",
        "any",
        "Make the list fit on its line, or set its items one per line (a stack or grid).",
        false,
    ),
    rule(
        "L084.heading-crowded-above",
        "warning",
        "A heading sits closer to the content above it than to its own content, so it \
         reads as part of what comes before.",
        "space above under 0.75 of the space below and under the heading size",
        "Leave about 1.5 to 2 times more space above a heading than below it.",
        false,
    ),
    rule(
        "L085.cramped-fraction-rows",
        "warning",
        "Stacked rows of grid or table cells that both hold built-up fractions are so \
         close that numerators and denominators crowd each other.",
        "0.5 text sizes of clear space",
        "Raise the grid's row-gutter (about 0.8 to 1em) or the cells' inset.",
        false,
    ),
    rule(
        "L086.loose-display-lines",
        "warning",
        "The lines of a multi-line title, name or headline are set with the leading of \
         running text, so they fall apart.",
        "line pitch over 1.35 sizes with over 0.35 sizes of clear space",
        "Set display text with its own leading in its own size (pitch 1.1 to 1.25).",
        false,
    ),
    rule(
        "L087.drifting-subtitle",
        "warning",
        "Smaller text under a large line (a name, a time, a product name) starts so far \
         below it, and no closer to it than to what follows, that it drifts away.",
        "1.3 sizes of the large line",
        "Keep the two in one stack or block about 0.3 to 0.6 sizes apart; paragraph \
         spacing in em grows with the large text.",
        false,
    ),
    rule(
        "L088.wrapped-label",
        "warning",
        "A short label in the first column of a grid or table wraps onto two lines while \
         the other labels of its column fit on one.",
        "3 words or 28 characters",
        "Widen the label column (auto or a wider fixed width), or shorten the label.",
        false,
    ),
    rule(
        "S001.negative-spacing",
        "warning",
        "Source lint: negative h() or v() spacing of at least 0.5em.",
        "from 0.5em or 5pt",
        "Remove the negative spacing and let layout place the content.",
        false,
    ),
    rule(
        "S002.absolute-placement",
        "warning",
        "Source lint: place() without float, or move(), outside page backgrounds, \
         headers, footers and fixed-size boxes or shapes (small drawing areas).",
        "any, unless labelled <allow-overlap>",
        "Keep content in the flow or use place(float: true).",
        false,
    ),
    rule(
        "S003.transform-without-reflow",
        "warning",
        "Source lint: rotate(), scale() or skew() without reflow: true.",
        "any",
        "Add reflow: true.",
        true,
    ),
    rule(
        "S004.disabled-protection",
        "warning",
        "Source lint: rules that disable orphan/widow costs or sticky headings.",
        "any",
        "Remove the rule and keep Typst's defaults.",
        true,
    ),
    rule(
        "S005.math-letter-call",
        "error",
        "Source lint: a one-letter function the document defines, written as f(..) inside          math, where it prints the letter and brackets instead of calling the function.",
        "any",
        "Call it with #f(..), or give the function a longer name.",
        false,
    ),
];

/// Typst's own diagnostics are reported with these rule names.
pub const TYPST_RULES: &[&str] = &[
    "typst.compile.error",
    "typst.compile.warning",
    "typst.pdf.error",
    "typst.pdf.warning",
];

#[cfg(test)]
mod tests {
    use super::CATALOG;

    #[test]
    fn every_rule_is_documented() {
        use crate::*;
        let used = [
            rules::PAGE_OVERFLOW,
            rules::CONTAINER_OVERFLOW,
            rules::MARGIN_INTRUSION,
            rules::MARGINAL_OVERFLOW,
            rules::LINE_OVERFLOW,
            rules::LOOSE_LINE,
            overlap::OVERLAP,
            whitespace::PAGE_UNDERFILL,
            resources::MISSING_GLYPH,
            resources::IMAGE_RESOLUTION,
            typography::ORPHAN,
            typography::WIDOW,
            typography::HYPHEN_LADDER,
            typography::HYPHEN_AT_BREAK,
            typography::STRANDED_HEADING,
            spacing::SPACING,
            structure::FIGURE_DISTANCE,
            structure::NUMBER_ALIGNMENT,
            structure::FOOTNOTE_PAGE,
            structure::NARROW_COLUMN,
            pagecheck::COLUMN_IMBALANCE,
            pagecheck::LINE_LENGTH,
            pagecheck::EMPTY_PAGE,
            quality::MIN_SIZE,
            quality::ASPECT,
            quality::RUNT,
            quality::LAST_PAGE,
            quality::SPLIT_FIGURE,
            quality::PAGE_FRAME,
            grid::OFF_GRID,
            typography::UNEVEN_LEADING,
            rules::COLUMN_OVERFLOW,
            rhythm::LEADING,
            rhythm::TOP_GAP,
            rhythm::ROW_BASELINES,
            rhythm::EQUATION_NUMBER,
            rhythm::WRAPPED_CODE,
            rhythm::CRAMPED,
            boxes::HOLLOW_BOX,
            boxes::TOUCHING_CELLS,
            structure::MIXED_PRECISION,
            typography::HYPHENATED_DISPLAY,
            pagecheck::EMPTY_COLUMN,
            composition::CENTERED_PARAGRAPH,
            composition::FIGURE_SPACING,
            composition::LOPSIDED_COLUMNS,
            contrast::LOW_CONTRAST,
            structure::RAGGED_COLUMN,
            marks::BROKEN_ADDRESS,
            marks::LINE_THROUGH_TEXT,
            typography::HYPHENATED_LABEL,
            typography::LONELY_TITLE_WORD,
            marks::MARKUP_LEAK,
            composition::TITLE_PROXIMITY,
            composition::PARAGRAPH_SPACING,
            composition::UNSEPARATED_TERMS,
            composition::SPARSE_BARS,
            marks::MALFORMED_NUMBER,
            boxes::CELL_PADDING,
            boxes::DOUBLE_RULE,
            boxes::BOX_PADDING,
            boxes::DANGLING_SEPARATOR,
            composition::CHART_LABELS,
            titles::DETACHED_SUBTITLE,
            titles::DETACHED_UNDERLINE,
            titles::FLOATING_ARROW,
            edges::PRINT_EDGE,
            titles::DROP_CAP_HOLE,
            boxes::UNEVEN_BOXES,
            titles::PAIR_SPACING,
            titles::DETACHED_LABEL,
            titles::TEXT_ON_RULE,
            titles::CROWDED_SUBTITLE,
            titles::DETACHED_FIGURE_CAPTION,
            boxes::OFF_CENTRE_PANEL,
            edges::PAGE_LABEL,
            boxes::SPREAD_BANNER,
            whitespace::NEAR_EMPTY_PAGE,
            overlap::BOX_OVER_TEXT,
            boxes::SPLIT_BOX,
            proximity::CLIPPED_TEXT,
            proximity::STRANDED_SEPARATOR,
            proximity::CROWDED_HEADING,
            proximity::CRAMPED_FRACTIONS,
            proximity::LOOSE_DISPLAY,
            proximity::DRIFTING_SUBTITLE,
            proximity::WRAPPED_LABEL,
            lint::NEGATIVE_SPACING,
            lint::ABSOLUTE_PLACEMENT,
            lint::NO_REFLOW,
            lint::DISABLED_PROTECTION,
            lint::MATH_LETTER_CALL,
        ];
        for id in used {
            assert!(CATALOG.iter().any(|r| r.id == id), "{id} is not in the catalog");
        }
        assert_eq!(used.len(), CATALOG.len(), "catalog lists unused rules");
    }

    #[test]
    fn texts_are_single_line() {
        for rule in CATALOG {
            for text in [rule.checks, rule.threshold, rule.repair] {
                assert!(!text.contains('\n'), "{}: {text:?}", rule.id);
            }
        }
    }

    #[test]
    fn ids_are_unique_and_ordered() {
        for pair in CATALOG.windows(2) {
            assert!(pair[0].id < pair[1].id, "{} !< {}", pair[0].id, pair[1].id);
        }
    }
}
