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
        "The last column ends far above the others on a non-final page.",
        "3 lines",
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
        "A line in a paragraph sits much further below the previous one than usual, \n         typically because of tall inline content.",
        "25% above the paragraph's median pitch, plus 1pt",
        "Move tall inline content into a display block, or reduce it.",
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
        ];
        for id in used {
            assert!(CATALOG.iter().any(|r| r.id == id), "{id} is not in the catalog");
        }
        assert_eq!(used.len(), CATALOG.len(), "catalog lists unused rules");
    }

    #[test]
    fn ids_are_unique_and_ordered() {
        for pair in CATALOG.windows(2) {
            assert!(pair[0].id < pair[1].id, "{} !< {}", pair[0].id, pair[1].id);
        }
    }
}
