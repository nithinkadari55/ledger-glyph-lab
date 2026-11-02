use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, Line, round1};
use crate::world::LabWorld;

pub const ORPHAN: &str = "L010.orphan";
pub const WIDOW: &str = "L011.widow";
pub const HYPHEN_LADDER: &str = "L012.hyphen-ladder";
pub const HYPHEN_AT_BREAK: &str = "L013.hyphen-at-page-break";
pub const STRANDED_HEADING: &str = "L014.stranded-heading";

/// The most consecutive lines that may end in an inserted hyphen. Convention.
const MAX_HYPHEN_RUN: usize = 2;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for lines in paragraphs(geometry) {
        breaks(world, &lines, &mut out);
        hyphen_ladder(world, &lines, &mut out);
    }
    for lines in cell_texts(geometry) {
        hyphen_ladder(world, &lines, &mut out);
    }
    stranded_headings(world, geometry, &mut out);
    out.extend(uneven_leading(world, geometry));
    out.extend(hyphenated_display(world, geometry));
    out.extend(hyphenated_labels(world, geometry));
    out.extend(lonely_title_words(world, geometry));
    out
}

pub const LONELY_TITLE_WORD: &str = "L052.lonely-title-word";

/// The number of words on a line.
fn line_words(geometry: &Geometry, line: &Line) -> usize {
    let b = &line.bbox;
    let mut items: Vec<_> = geometry
        .items
        .iter()
        .filter(|i| i.page == line.page && i.kind == crate::geom::ItemKind::Text)
        .filter(|i| {
            let cx = (i.bbox.x0 + i.bbox.x1) / 2.0;
            let cy = (i.bbox.y0 + i.bbox.y1) / 2.0;
            cy >= b.y0 && cy <= b.y1 && cx >= b.x0 - 1.0 && cx <= b.x1 + 1.0
        })
        .collect();
    items.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
    let text: String = items.iter().filter_map(|i| i.text.as_deref()).collect();
    text.split_whitespace().count()
}

/// Titles and headings whose last line holds a single word under a full line,
/// as in "Moving Patient Records to the" followed by "Cloud".
fn lonely_title_words(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let lines = &geometry.lines;
    for index in 1..lines.len() {
        let (above, last) = (&lines[index - 1], &lines[index]);
        let step = last.bbox.y1 - above.bbox.y1;
        if above.page != last.page
            || above.container != last.container
            || above.hyphenated
            || step <= 0.0
            || step > 2.0 * above.bbox.height()
        {
            continue;
        }
        let Some(size) = crate::spacing::line_size(geometry, last) else { continue };
        // Only the last line of a block counts; text of another size below
        // (a byline, the body) starts a new block.
        if let Some(next) = lines.get(index + 1)
            && next.page == last.page
            && next.container == last.container
            && next.bbox.y1 > last.bbox.y1
            && next.bbox.y1 - last.bbox.y1 <= 2.0 * last.bbox.height()
            && crate::spacing::line_size(geometry, next)
                .is_some_and(|next_size| (next_size - size).abs() < 0.1 * size)
        {
            continue;
        }
        if size < DISPLAY_TEXT_PT
            || line_words(geometry, last) != 1
            || line_words(geometry, above) < 3
        {
            continue;
        }
        let mut diagnostic = line_diagnostic(
            world,
            LONELY_TITLE_WORD,
            last,
            format!(
                "the last line of {}pt display text on page {} holds a single word",
                round1(size),
                last.page + 1
            ),
            "balance the lines of the title: insert a line break earlier (\\ or \
             linebreak()), narrow its block, or reword it",
        );
        diagnostic.elem = last.elem.map(|e| geometry.elements[e].name.into());
        out.push(diagnostic);
    }
    out
}

pub const HYPHENATED_LABEL: &str = "L051.hyphenated-label";

/// A text block of at most this many lines in a measure narrower than
/// `MAX_LABEL_MEASURE_PT` is a label: chart labels, stat captions, narrow
/// table cells. Breaking one of its words reads as a mistake.
const MAX_LABEL_LINES: usize = 3;
const MAX_LABEL_MEASURE_PT: f64 = 120.0;

fn hyphenated_labels(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let same_block = |a: &Line, b: &Line| {
        a.page == b.page
            && a.container == b.container
            && (a.bbox.x0 - b.bbox.x0).abs() < 1.0
            && (a.available - b.available).abs() < 1.0
    };
    let lines = &geometry.lines;
    let mut out = Vec::new();
    let mut start = 0;
    while start < lines.len() {
        let mut end = start + 1;
        while end < lines.len() && same_block(&lines[start], &lines[end]) {
            end += 1;
        }
        let block = &lines[start..end];
        start = end;
        if block.len() > MAX_LABEL_LINES || block[0].available >= MAX_LABEL_MEASURE_PT {
            continue;
        }
        let Some(line) = block.iter().find(|l| l.hyphenated) else { continue };
        // Large text is reported by L042.
        if crate::spacing::line_size(geometry, line).is_some_and(|s| s >= DISPLAY_TEXT_PT) {
            continue;
        }
        let mut diagnostic = line_diagnostic(
            world,
            HYPHENATED_LABEL,
            line,
            format!(
                "a {}-line label {}pt wide on page {} breaks a word with a hyphen",
                block.len(),
                round1(line.available),
                line.page + 1
            ),
            "widen the label's column, shorten the label, or set text(hyphenate: false) \
             for labels so their words stay whole",
        );
        diagnostic.measured = Some(round1(line.available));
        diagnostic.limit = Some(MAX_LABEL_MEASURE_PT);
        diagnostic.unit = Some("pt");
        out.push(diagnostic);
    }
    out
}

pub const HYPHENATED_DISPLAY: &str = "L042.hyphenated-display-text";

/// Text at least this large is display text (titles, headings, pull quotes),
/// which is not hyphenated. Convention.
const DISPLAY_TEXT_PT: f64 = 14.0;

fn hyphenated_display(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    geometry
        .lines
        .iter()
        .filter(|line| line.hyphenated)
        .filter_map(|line| {
            let size = crate::spacing::line_size(geometry, line)?;
            (size >= DISPLAY_TEXT_PT).then(|| {
                let mut diagnostic = line_diagnostic(
                    world,
                    HYPHENATED_DISPLAY,
                    line,
                    format!(
                        "display text set at {}pt is hyphenated on page {}",
                        round1(size),
                        line.page + 1
                    ),
                    "turn hyphenation off for titles, headings and quotes (set \
                     text(hyphenate: false) in their show rule) and break the line by \
                     wording or width instead",
                );
                diagnostic.measured = Some(round1(size));
                diagnostic.limit = Some(DISPLAY_TEXT_PT);
                diagnostic.unit = Some("pt");
                diagnostic
            })
        })
        .collect()
}

/// The lines of each paragraph in document order. Lines that do not belong to
/// a paragraph element are skipped.
/// The paragraph that an element belongs to: the element itself if it is a
/// paragraph, or its nearest paragraph ancestor. Lines may report an inline
/// element such as an equation or a link that starts at or before them.
pub fn paragraph_of(geometry: &Geometry, mut cursor: Option<usize>) -> Option<usize> {
    while let Some(index) = cursor {
        match geometry.elements[index].name {
            "par" => return Some(index),
            // Footnote entries are laid out in their own area. Typst nests them
            // under the paragraph that holds the marker, but their lines are
            // not part of it.
            "entry" => return None,
            _ => {}
        }
        cursor = geometry.elements[index].parent;
    }
    None
}

fn paragraphs(geometry: &Geometry) -> Vec<Vec<&Line>> {
    let mut out: Vec<Vec<&Line>> = Vec::new();
    let mut current = None;
    for line in &geometry.lines {
        let Some(elem) = paragraph_of(geometry, line.elem) else {
            current = None;
            continue;
        };
        if current != Some(elem) {
            out.push(Vec::new());
            current = Some(elem);
        }
        out.last_mut().unwrap().push(line);
    }
    out
}

/// Text set directly in a table or grid cell has no paragraph element; its
/// lines are grouped by the cell instead.
fn cell_texts(geometry: &Geometry) -> Vec<Vec<&Line>> {
    let mut out: Vec<Vec<&Line>> = Vec::new();
    let mut current = None;
    for line in &geometry.lines {
        let cell = line
            .elem
            .filter(|_| paragraph_of(geometry, line.elem).is_none())
            .filter(|e| geometry.elements[*e].name == "cell");
        let Some(cell) = cell else {
            current = None;
            continue;
        };
        if current != Some(cell) {
            out.push(Vec::new());
            current = Some(cell);
        }
        out.last_mut().unwrap().push(line);
    }
    out
}

fn breaks(world: &LabWorld, lines: &[&Line], out: &mut Vec<Diagnostic>) {
    for pair in lines.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        if before.page == after.page {
            continue;
        }
        let on_before = lines.iter().filter(|l| l.page == before.page).count();
        let on_after = lines.iter().filter(|l| l.page == after.page).count();
        if on_before == 1 && lines.len() > 1 {
            out.push(line_diagnostic(
                world,
                ORPHAN,
                before,
                format!(
                    "the first line of a paragraph is left alone at the bottom of page {}",
                    before.page + 1
                ),
                "keep at least two lines together (the default par costs prevent this \
                 unless overridden), or adjust the preceding content",
            ));
        }
        if on_after == 1 && lines.len() > 1 {
            out.push(line_diagnostic(
                world,
                WIDOW,
                after,
                format!(
                    "the last line of a paragraph is left alone at the top of page {}",
                    after.page + 1
                ),
                "keep at least two lines together (the default par costs prevent this \
                 unless overridden), or adjust the paragraph length",
            ));
        }
        if before.hyphenated {
            out.push(line_diagnostic(
                world,
                HYPHEN_AT_BREAK,
                before,
                format!(
                    "a hyphenated word is split across pages {} and {}",
                    before.page + 1,
                    after.page + 1
                ),
                "rephrase the line or adjust the preceding content so the word is not \
                 split across the page break",
            ));
        }
    }
}

fn hyphen_ladder(world: &LabWorld, lines: &[&Line], out: &mut Vec<Diagnostic>) {
    let mut run = 0;
    for (i, line) in lines.iter().enumerate() {
        run = if line.hyphenated { run + 1 } else { 0 };
        let ends_run = !lines.get(i + 1).is_some_and(|next| next.hyphenated);
        if run > MAX_HYPHEN_RUN && ends_run {
            let first = lines[i + 1 - run];
            let mut diagnostic = line_diagnostic(
                world,
                HYPHEN_LADDER,
                first,
                format!(
                    "{run} consecutive lines end with a hyphen on page {}",
                    first.page + 1
                ),
                "rephrase the paragraph or adjust the text width so fewer consecutive \
                 lines are hyphenated",
            );
            diagnostic.measured = Some(run as f64);
            diagnostic.limit = Some(MAX_HYPHEN_RUN as f64);
            diagnostic.unit = Some("lines");
            out.push(diagnostic);
        }
    }
}

fn stranded_headings(world: &LabWorld, geometry: &Geometry, out: &mut Vec<Diagnostic>) {
    for (index, element) in geometry.elements.iter().enumerate() {
        if element.name != "heading" {
            continue;
        }
        let Some(last) = element.boxes.last() else { continue };
        let page = last.page;
        // In a multi-column layout only the heading's own column follows it;
        // content in a later column of the page is a continuation.
        let column = geometry
            .items
            .iter()
            .filter(|i| i.page == page && descends_from(geometry, i.elem, index))
            .filter_map(|i| i.column)
            .next_back();
        let in_column = |x0: f64, x1: f64, item_column: Option<usize>| match column {
            None => true,
            Some(c) => item_column.map_or_else(
                || {
                    let bbox = &geometry.columns[c].bbox;
                    x0 >= bbox.x0 - 1.0 && x1 <= bbox.x1 + 1.0
                },
                |k| k == c,
            ),
        };
        let below = geometry.items.iter().any(|item| {
            item.page == page
                && geometry.is_body(item.region)
                && item.bbox.y0 >= last.bbox.y1
                && in_column(item.bbox.x0, item.bbox.x1, item.column)
                && !descends_from(geometry, item.elem, index)
        });
        let continues = geometry.items.iter().any(|item| {
            item.page > page
                || column.is_some_and(|c| {
                    item.page == page
                        && item.column.is_some_and(|k| {
                            k != c
                                && geometry.columns[k].bbox.x0
                                    > geometry.columns[c].bbox.x0
                        })
                })
        });
        if !continues {
            continue;
        }
        let lines_below: Vec<_> = geometry
            .lines
            .iter()
            .filter(|l| {
                l.page == page
                    && geometry.is_body(l.region)
                    && l.bbox.y0 >= last.bbox.y1 - 0.5
                    && in_column(l.bbox.x0, l.bbox.x1, None)
                    && !descends_from(geometry, l.elem, index)
            })
            .collect();
        let message = match lines_below.as_slice() {
            _ if !below => format!(
                "heading is the last content on page {}; its section starts on the next \
                 page",
                page + 1
            ),
            [only]
                if !geometry.items.iter().any(|item| {
                    item.page == page
                        && geometry.is_body(item.region)
                        && item.bbox.y0 > only.bbox.y1 + 0.5
                        && in_column(item.bbox.x0, item.bbox.x1, item.column)
                }) =>
            {
                format!(
                    "heading is followed by only one line of its section on page {}",
                    page + 1
                )
            }
            _ => continue,
        };
        let mut diagnostic =
            Diagnostic::new(STRANDED_HEADING, Severity::Warning, message);
        diagnostic.loc = diag::resolve_span(world, element.span);
        diagnostic.elem = Some("heading".into());
        diagnostic.label = element.label.clone();
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = Some(last.bbox.rounded());
        diagnostic.hints.push(
            "keep headings with the following content (do not disable the default \
             sticky behaviour), or adjust the preceding content"
                .into(),
        );
        out.push(diagnostic);
    }
}

fn descends_from(
    geometry: &Geometry,
    mut cursor: Option<usize>,
    ancestor: usize,
) -> bool {
    while let Some(index) = cursor {
        if index == ancestor {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

fn line_diagnostic(
    world: &LabWorld,
    rule: &'static str,
    line: &Line,
    message: String,
    hint: &str,
) -> Diagnostic {
    let mut out = Diagnostic::new(rule, Severity::Warning, message);
    out.loc = diag::resolve_span(world, line.span);
    out.elem = Some("par".into());
    out.page = Some(line.page + 1);
    out.bbox = Some(line.bbox.rounded());
    out.hints.push(hint.into());
    out
}

pub const UNEVEN_LEADING: &str = "L031.uneven-line-spacing";

/// How much taller than the paragraph's usual line pitch a line may be.
const MAX_PITCH_GROWTH: f64 = 1.25;

/// Lines whose distance to the previous line is much larger than the
/// paragraph's usual line pitch, typically because a tall inline element
/// (a fraction, a large symbol or an inline image) widens the line.
pub fn uneven_leading(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for lines in paragraphs(geometry) {
        let columns: Vec<Vec<&Line>> = lines
            .chunk_by(|a, b| a.page == b.page && (a.bbox.x0 - b.bbox.x0).abs() < 0.5)
            .map(<[_]>::to_vec)
            .collect();
        for run in columns {
            if run.len() < 3 {
                continue;
            }
            let pitches: Vec<f64> =
                run.windows(2).map(|w| w[1].bbox.y0 - w[0].bbox.y0).collect();
            let mut sorted = pitches.clone();
            sorted.sort_by(f64::total_cmp);
            let usual = sorted[sorted.len() / 2];
            if usual <= 0.0 {
                continue;
            }
            let Some((index, &worst)) =
                pitches.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1))
            else {
                continue;
            };
            if worst <= usual * MAX_PITCH_GROWTH + 1.0 {
                continue;
            }
            let taller = run[index + 1];
            let mut diagnostic = line_diagnostic(
                world,
                UNEVEN_LEADING,
                taller,
                format!(
                    "a line on page {} sits {}pt below the previous one, but lines in this \
                     paragraph are usually {}pt apart",
                    taller.page + 1,
                    round1(worst),
                    round1(usual)
                ),
                "move tall inline content (fractions, large symbols, images) into its \
                 own display block, or reduce its size",
            );
            diagnostic.measured = Some(round1(worst));
            diagnostic.limit = Some(round1(usual * MAX_PITCH_GROWTH + 1.0));
            diagnostic.unit = Some("pt");
            out.push(diagnostic);
        }
    }
    out
}
