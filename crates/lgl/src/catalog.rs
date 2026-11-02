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
        "0.5pt (text: 0.35em)",
        "Give the container more room (auto, fr, %), let content wrap, or clip on \
         purpose.",
        true,
    ),
    rule(
        "L003.margin-intrusion",
        "error",
        "Body content extends into the page margin.",
        "0.5pt (text: 0.35em for punctuation overhang)",
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
        "0.5pt (text: 0.35em)",
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
        "A figure is set off from the text above or below it by barely more than the \
         line spacing, so its caption reads as part of the paragraph.",
        "0.3em beyond the line pitch",
        "Give figures at least paragraph spacing above and below.",
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
         \"www.example.\" followed by \"com\".",
        "any split address",
        "Keep the address on one line: shorten the text around it or wrap it in box().",
        false,
    ),
    rule(
        "L050.line-through-text",
        "warning",
        "A thin line (rule, tick, gridline) runs through text, or a horizontal line \
         touches its ink, such as a heading rule set right under the descenders. \
         Underline, strike, overline, highlight and content labelled <allow-overlap> \
         are exempt.",
        "a fifth of the text height or width; 0.08em clearance",
        "End lines before labels, move the text, or give labels a background.",
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
            lint::NEGATIVE_SPACING,
            lint::ABSOLUTE_PLACEMENT,
            lint::NO_REFLOW,
            lint::DISABLED_PROTECTION,
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
