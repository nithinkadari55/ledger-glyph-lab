//! Checks of how parts of a page group together, measured on the ink:
//! text cut off by a clip, separators stranded at line ends, headings closer
//! to the content above them than to their own, rows of fractions that crowd
//! each other, loosely set display lines, smaller text that drifts away from
//! the large line it belongs to, and short labels that wrap.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use typst::syntax::Span;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Element, Geometry, Item, ItemKind, Line, round1};
use crate::world::LabWorld;

pub const CLIPPED_TEXT: &str = "L082.clipped-text";
pub const STRANDED_SEPARATOR: &str = "L083.stranded-separator";
pub const CROWDED_HEADING: &str = "L084.heading-crowded-above";
pub const CRAMPED_FRACTIONS: &str = "L085.cramped-fraction-rows";
pub const LOOSE_DISPLAY: &str = "L086.loose-display-lines";
pub const DRIFTING_SUBTITLE: &str = "L087.drifting-subtitle";
pub const WRAPPED_LABEL: &str = "L088.wrapped-label";

/// Text may reach this far beyond the box that clips it (glyph outlines and
/// side bearings), in points or font sizes, whichever is larger.
const CLIP_TOLERANCE_PT: f64 = 0.5;
const CLIP_TOLERANCE_EM: f64 = 0.1;

/// Characters that separate the items of an inline list.
const SEPARATORS: &[char] = &['·', '|', '•', '⋅', '∙', '◦', '‧'];

/// A heading is crowded from above when the space above it is less than this
/// share of the space below it, and less than its own size. Measured on 1,900
/// headings of 440 generated documents: only headings that read as part of the
/// content above fall below both.
const MAX_HEADING_ABOVE_SHARE: f64 = 0.75;

/// Rows of built-up fractions need at least this much clear space between
/// their ink, in text sizes. Measured on 440 generated documents: answer grids
/// at 0.27 to 0.48 read as one jammed block.
const MIN_FRACTION_ROW_GAP_EM: f64 = 0.5;

/// Display lines (titles, names, headlines) set further apart than this pitch
/// and with this much clear space between their ink, in text sizes, fall
/// apart into separate lines. Measured on 203 pairs of display lines: the
/// median pitch is 1.21 and the median clear space 0.26; script faces with
/// deep swashes keep less than 0.35 clear even at a looser pitch.
const MAX_DISPLAY_PITCH_EM: f64 = 1.35;
const MAX_DISPLAY_CLEAR_EM: f64 = 0.35;

/// Smaller text starting further than this many sizes below a large line,
/// and no closer to it than to what follows, has drifted away from it.
/// Measured on 440 generated documents: gaps of 0.9 to 1.2 sizes still read
/// as one group; from 1.3 the smaller text reads as the start of what follows.
const MAX_SUBTITLE_GAP_EM: f64 = 1.3;

/// Text at least this large, and this many times the body size, is display
/// text.
const DISPLAY_PT: f64 = 14.0;
const DISPLAY_RATIO: f64 = 1.4;

/// A label of at most this many words and characters is short enough to
/// stay on one line.
const SHORT_LABEL_WORDS: usize = 3;
const SHORT_LABEL_CHARS: usize = 28;

/// The label of the agent kit's section headings.
const KIT_HEADING: &str = "kit-section-heading";

pub fn run(world: &LabWorld, geometry: &Geometry, earlier: &[Diagnostic]) -> Vec<Diagnostic> {
    let body = body_size(geometry);
    let tight = rows(geometry, |_| 1.0);
    let mut out = clipped_text(world, geometry);
    out.extend(stranded_separators(world, geometry, &tight));
    out.extend(crowded_headings(world, geometry, &tight, body));
    out.extend(cramped_fraction_rows(world, geometry));
    out.extend(loose_display_lines(world, geometry, &tight, body));
    out.extend(drifting_subtitles(world, geometry, body, earlier));
    out.extend(wrapped_labels(world, geometry, &tight));
    out
}

// --- shared measurements ------------------------------------------------------

/// A line of text with the items it holds and their ink.
struct Row {
    line: usize,
    page: usize,
    items: Vec<usize>,
    ink: BBox,
    size: f64,
    text: String,
}

fn nonblank(item: &Item) -> bool {
    item.kind == ItemKind::Text
        && item.text.as_deref().is_some_and(|text| !text.trim().is_empty())
}

/// The text lines of the document, each with the text items whose middle lies
/// within the line's box (widened by `tolerance` above and below).
fn rows(geometry: &Geometry, tolerance: impl Fn(&Line) -> f64) -> Vec<Row> {
    let mut out = Vec::new();
    for (index, line) in geometry.lines.iter().enumerate() {
        let b = &line.bbox;
        let tol = tolerance(line);
        let mut items: Vec<usize> = geometry
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                let middle = (item.bbox.y0 + item.bbox.y1) / 2.0;
                nonblank(item)
                    && item.page == line.page
                    && middle >= b.y0 - tol
                    && middle <= b.y1 + tol
                    && item.bbox.x0 >= b.x0 - 1.0
                    && item.bbox.x1 <= b.x1 + 1.0
            })
            .map(|(k, _)| k)
            .collect();
        if items.is_empty() {
            continue;
        }
        items.sort_by(|a, b| geometry.items[*a].bbox.x0.total_cmp(&geometry.items[*b].bbox.x0));
        let ink = items
            .iter()
            .map(|&k| geometry.items[k].bbox)
            .reduce(|a, b| a.union(&b))
            .unwrap_or(*b);
        let size = items
            .iter()
            .filter_map(|&k| geometry.items[k].font_size)
            .fold(0.0, f64::max);
        let text = items
            .iter()
            .filter_map(|&k| geometry.items[k].text.as_deref())
            .collect::<String>();
        out.push(Row { line: index, page: line.page, items, ink, size, text });
    }
    out
}

/// The usual size of running text: the median size of longer text runs.
fn body_size(geometry: &Geometry) -> f64 {
    let mut sizes: Vec<f64> = geometry
        .items
        .iter()
        .filter(|i| i.kind == ItemKind::Text)
        .filter(|i| i.text.as_deref().is_some_and(|t| t.chars().count() > 20))
        .filter_map(|i| i.font_size)
        .collect();
    sizes.sort_by(f64::total_cmp);
    sizes.get(sizes.len() / 2).copied().unwrap_or(10.0)
}

fn ancestor(
    geometry: &Geometry,
    mut elem: Option<usize>,
    found: impl Fn(&Element) -> bool,
) -> Option<usize> {
    while let Some(index) = elem {
        let element = &geometry.elements[index];
        if found(element) {
            return Some(index);
        }
        elem = element.parent;
    }
    None
}

/// The innermost box with a fill or stroke around a container.
fn visible_box(geometry: &Geometry, mut cursor: Option<usize>) -> Option<usize> {
    while let Some(index) = cursor {
        if geometry.containers[index].visible {
            return Some(index);
        }
        cursor = geometry.containers[index].parent;
    }
    None
}

/// A thin horizontal rule.
fn is_rule(item: &Item) -> bool {
    item.kind == ItemKind::Shape && item.bbox.height() < 3.0 && item.bbox.width() >= 20.0
}

fn overlaps_x(a: &BBox, b: &BBox) -> bool {
    a.x0 < b.x1 && b.x0 < a.x1
}

fn short(text: &str) -> String {
    let text = text.trim();
    if text.chars().count() <= 40 {
        text.to_string()
    } else {
        format!("{}…", text.chars().take(39).collect::<String>())
    }
}

fn line_span(geometry: &Geometry, line: &Line) -> Span {
    geometry.span_for(line.span, line.elem)
}

fn item_span(geometry: &Geometry, item: &Item) -> Span {
    geometry.span_for(item.span, item.elem)
}

fn located(
    world: &LabWorld,
    mut diagnostic: Diagnostic,
    span: Span,
    page: usize,
    bbox: BBox,
) -> Diagnostic {
    diagnostic.loc = diag::resolve_span(world, span);
    diagnostic.page = Some(page + 1);
    diagnostic.bbox = Some(bbox.rounded());
    diagnostic
}

// --- L082: text cut off by a clip ---------------------------------------------

/// Text that a clipping box cuts off. A box clips to round its corners or to
/// crop a picture; text that runs past its edge is simply lost.
fn clipped_text(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut seen: HashSet<(usize, Span)> = HashSet::new();
    for item in &geometry.items {
        if !nonblank(item) {
            continue;
        }
        let Some(clip) = item.clip else { continue };
        let b = &item.bbox;
        let size = item.font_size.unwrap_or(10.0);
        let cut = (clip.x0 - b.x0).max(b.x1 - clip.x1).max(clip.y0 - b.y0).max(b.y1 - clip.y1);
        let limit = CLIP_TOLERANCE_PT.max(CLIP_TOLERANCE_EM * size);
        let span = item_span(geometry, item);
        if cut <= limit || !seen.insert((item.page, span)) {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            CLIPPED_TEXT,
            Severity::Error,
            format!(
                "\"{}\" on page {} runs {}pt past the edge of a box that clips its \
                 content, so that part of it is not printed",
                short(item.text.as_deref().unwrap_or_default()),
                item.page + 1,
                round1(cut)
            ),
        );
        diagnostic.measured = Some(round1(cut));
        diagnostic.limit = Some(round1(limit));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "give the text room inside the box (a smaller size, shorter text or a wider \
             box); a box clips to round its corners, so it never makes space for text"
                .into(),
        );
        out.push(located(world, diagnostic, span, item.page, *b));
    }
    out
}

// --- L083: a separator stranded at a line end -----------------------------------

/// An inline list joined by separators ("Bristol · +44 7700 900123 · ..") that
/// wraps, leaving a separator at the end or the start of a line.
fn stranded_separators(world: &LabWorld, geometry: &Geometry, rows: &[Row]) -> Vec<Diagnostic> {
    let by_line: BTreeMap<usize, &Row> = rows.iter().map(|r| (r.line, r)).collect();
    let mut out = Vec::new();
    for (k, pair) in geometry.lines.windows(2).enumerate() {
        let (a, b) = (&pair[0], &pair[1]);
        let (Some(ra), Some(rb)) = (by_line.get(&k), by_line.get(&(k + 1))) else {
            continue;
        };
        let (ba, bb) = (&a.bbox, &b.bbox);
        if a.page != b.page
            || a.container != b.container
            || bb.y0 <= ba.y0
            || bb.y0 - ba.y1 > 2.0 * ba.height() + 4.0
        {
            continue;
        }
        let aligned = (ba.x0 - bb.x0).abs() <= 40.0
            || (ba.x1 - bb.x1).abs() <= 40.0
            || ((ba.x0 + ba.x1) - (bb.x0 + bb.x1)).abs() <= 40.0;
        if !aligned {
            continue;
        }
        let end: Vec<char> = ra.text.trim_end().chars().collect();
        let start: Vec<char> = rb.text.trim_start().chars().collect();
        let at_end = end.len() >= 3
            && SEPARATORS.contains(&end[end.len() - 1])
            && end[end.len() - 2].is_whitespace();
        let at_start = start.len() >= 3
            && SEPARATORS.contains(&start[0])
            && start[1].is_whitespace();
        if !at_end && !at_start {
            continue;
        }
        let separator = if at_end { end[end.len() - 1] } else { start[0] };
        let diagnostic = Diagnostic::new(
            STRANDED_SEPARATOR,
            Severity::Warning,
            format!(
                "a list joined by \"{separator}\" wraps on page {}, leaving the separator at \
                 the {} of a line",
                a.page + 1,
                if at_end { "end" } else { "start" }
            ),
        );
        let mut diagnostic =
            located(world, diagnostic, line_span(geometry, a), a.page, ra.ink.union(&rb.ink));
        diagnostic.hints.push(
            "a list joined by separators must fit on its line: shorten it or make its text \
             smaller, or set its items one per line (a stack or grid) instead of letting \
             it wrap"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

// --- L084: a heading closer to the content above than to its own ----------------

/// A heading that sits closer to the content above it than to the content it
/// introduces, so it reads as the end of the previous section. Headings are
/// Typst headings, the kit's section headings, and single lines underlined by
/// a rule whose style recurs on other lines (a section label, not a subtitle).
fn crowded_headings(
    world: &LabWorld,
    geometry: &Geometry,
    rows: &[Row],
    body: f64,
) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let rules: Vec<usize> = (0..items.len()).filter(|&k| is_rule(&items[k])).collect();
    // The rule directly under a block of text, as wide as it.
    let rule_under = |page: usize, ink: &BBox, size: f64| {
        rules.iter().copied().find(|&r| {
            let b = &items[r].bbox;
            items[r].page == page
                && b.y0 - ink.y1 >= 0.0
                && b.y0 - ink.y1 <= 0.8 * size
                && b.x0 <= ink.x0 + 2.0
                && b.x1 >= ink.x1 - 2.0
        })
    };

    let mut groups: Vec<(bool, Vec<usize>)> = Vec::new();
    let mut explicit: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for (k, item) in items.iter().enumerate() {
        if !nonblank(item) {
            continue;
        }
        if let Some(heading) = ancestor(geometry, item.elem, |e| {
            e.name == "heading" || e.label.as_deref() == Some(KIT_HEADING)
        }) {
            explicit.entry((heading, item.page)).or_default().push(k);
        }
    }
    groups.extend(explicit.into_values().map(|group| (false, group)));
    for row in rows {
        let first = &items[row.items[0]];
        if row.text.trim().chars().count() > 60
            || ancestor(geometry, first.elem, |e| {
                ["table", "list", "enum", "terms", "figure", "heading"].contains(&e.name)
                    || e.label.as_deref() == Some(KIT_HEADING)
            })
            .is_some()
            || row.size > 2.0 * body
        {
            continue;
        }
        let lb = &geometry.lines[row.line].bbox;
        let near = geometry.lines.iter().enumerate().any(|(m, other)| {
            let o = &other.bbox;
            m != row.line
                && other.page == row.page
                && overlaps_x(o, lb)
                && ((0.0..0.8 * row.size).contains(&(lb.y0 - o.y1))
                    || (0.0..0.8 * row.size).contains(&(o.y0 - lb.y1)))
        });
        if !near && rule_under(row.page, &row.ink, row.size).is_some() {
            groups.push((true, row.items.clone()));
        }
    }

    // A section label's style recurs on other rows; a subtitle's does not.
    let style = |group: &[usize]| {
        let size = group.iter().filter_map(|&k| items[k].font_size).fold(0.0, f64::max);
        let color = items[group[0]]
            .color
            .map(|c| c.map(|v| (f64::from(v) * 100.0).round() as i64));
        ((size * 10.0).round() as i64, color)
    };
    let mut recurring: BTreeMap<_, BTreeSet<(usize, i64)>> = BTreeMap::new();
    for (_, group) in &groups {
        let top = group.iter().map(|&k| items[k].bbox.y0).fold(f64::INFINITY, f64::min);
        recurring
            .entry(style(group))
            .or_default()
            .insert((items[group[0]].page, top.round() as i64));
    }

    let mut out = Vec::new();
    for (underlined, group) in &groups {
        if *underlined && recurring[&style(group)].len() < 2 {
            continue;
        }
        let head = &items[group[0]];
        let page = head.page;
        let size = group.iter().filter_map(|&k| items[k].font_size).fold(0.0, f64::max);
        let Some(mut h) = group.iter().map(|&k| items[k].bbox).reduce(|a, b| a.union(&b))
        else {
            continue;
        };
        let text_ink = h;
        let under = rule_under(page, &text_ink, size);
        if let Some(r) = under {
            h.y1 = items[r].bbox.y1;
        }
        // The heading's column: the lines that hold it.
        let (mut cx0, mut cx1) = (h.x0, h.x1);
        for line in geometry.lines.iter().filter(|l| l.page == page) {
            let b = &line.bbox;
            let holds = group.iter().any(|&k| {
                let i = &items[k].bbox;
                let middle = (i.y0 + i.y1) / 2.0;
                b.y0 - 1.0 <= middle
                    && middle <= b.y1 + 1.0
                    && b.x0 - 1.0 <= i.x0
                    && i.x1 <= b.x1 + 1.0
            });
            if holds {
                cx0 = cx0.min(b.x0);
                cx1 = cx1.max(b.x1);
            }
        }
        let others: Vec<&Item> = items
            .iter()
            .enumerate()
            .filter(|(k, i)| {
                i.kind != ItemKind::Link
                    && i.page == page
                    && !group.contains(k)
                    && Some(*k) != under
                    && i.region == head.region
                    && (head.column.is_none() || i.column.is_none() || i.column == head.column)
                    && i.bbox.x0 < cx1
                    && i.bbox.x1 > cx0
                    && !(i.bbox.y0 <= h.y0 + 0.5 && i.bbox.y1 >= h.y1 - 0.5)
            })
            .map(|(_, i)| i)
            .collect();
        let Some(above) = others
            .iter()
            .filter(|i| i.bbox.y1 <= h.y0 + 0.5)
            .max_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1))
        else {
            continue;
        };
        let Some(below) = others
            .iter()
            .filter(|i| i.bbox.y0 >= h.y1 - 0.5)
            .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0))
        else {
            continue;
        };
        let gap_above = h.y0 - above.bbox.y1;
        let gap_below = below.bbox.y0 - h.y1;
        // A divider rule above a heading belongs close to it.
        if is_rule(above) {
            continue;
        }
        // Writing lines below a heading (a notes area) leave space to write in.
        if is_rule(below) {
            let mut column: Vec<f64> = rules
                .iter()
                .map(|&r| &items[r])
                .filter(|r| {
                    r.page == page
                        && (r.bbox.width() - below.bbox.width()).abs() < 3.0
                        && r.bbox.y0 >= below.bbox.y0 - 0.5
                })
                .map(|r| r.bbox.y0)
                .collect();
            column.sort_by(f64::total_cmp);
            if column.len() >= 3
                && ((column[1] - column[0]) - (column[2] - column[1])).abs() <= 1.0
            {
                continue;
            }
        }
        // A kicker: a short, smaller line standing on its own right above.
        if let Some(kicker) = others
            .iter()
            .find(|i| i.kind == ItemKind::Text && (i.bbox.y1 - above.bbox.y1).abs() < 0.5)
        {
            let kicker_size = kicker.font_size.unwrap_or_default();
            let text: String = others
                .iter()
                .filter(|i| {
                    i.kind == ItemKind::Text
                        && i.bbox.y1 <= h.y0 + 0.5
                        && (i.bbox.y1 - kicker.bbox.y1).abs() < 1.0
                })
                .filter_map(|i| i.text.as_deref())
                .collect();
            let over = others
                .iter()
                .filter(|i| i.bbox.y1 <= kicker.bbox.y0 + 0.5)
                .map(|i| i.bbox.y1)
                .fold(f64::NEG_INFINITY, f64::max);
            if kicker_size <= 0.8 * size
                && text.trim().chars().count() <= 40
                && (over == f64::NEG_INFINITY || kicker.bbox.y0 - over > 2.0 * gap_above)
            {
                continue;
            }
        }
        if gap_below <= 0.0
            || gap_above >= MAX_HEADING_ABOVE_SHARE * gap_below
            || gap_above >= size
        {
            continue;
        }
        let mut ordered = group.clone();
        ordered.sort_by(|a, b| {
            items[*a]
                .bbox
                .y0
                .total_cmp(&items[*b].bbox.y0)
                .then(items[*a].bbox.x0.total_cmp(&items[*b].bbox.x0))
        });
        let text: String = ordered.iter().filter_map(|&k| items[k].text.as_deref()).collect();
        let mut diagnostic = Diagnostic::new(
            CROWDED_HEADING,
            Severity::Warning,
            format!(
                "the heading \"{}\" on page {} sits {}pt below the content above it but \
                 {}pt above its own content, so it reads as part of what comes before",
                short(&text),
                page + 1,
                round1(gap_above),
                round1(gap_below)
            ),
        );
        diagnostic.measured = Some(round1(gap_above));
        diagnostic.limit = Some(round1(MAX_HEADING_ABOVE_SHARE * gap_below));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "leave clearly more space above a heading than below it, about 1.5 to 2 times \
             (e.g. a larger block(above: ..) on the heading, or less space below it)"
                .into(),
        );
        out.push(located(world, diagnostic, item_span(geometry, head), page, h));
    }
    out
}

// --- L085: rows of fractions crowding each other -------------------------------

/// Stacked rows of separate items (grid or table cells) that both hold
/// built-up fractions, set so close that numerators and denominators crowd.
fn cramped_fraction_rows(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let items = &geometry.items;
    // Fractions reach above and below a line's box, so take in more of them.
    let all = rows(geometry, |line| 0.5 * line.bbox.height() + 1.0);
    let mut fractions: Vec<(&Row, usize)> = Vec::new();
    for row in &all {
        let Some(cell) = ancestor(geometry, items[row.items[0]].elem, |e| e.name == "cell")
        else {
            continue;
        };
        let lb = &geometry.lines[row.line].bbox;
        let small = |i: &Item| i.font_size.unwrap_or_default() < 0.85 * row.size;
        let has_fraction = items.iter().any(|bar| {
            let b = &bar.bbox;
            bar.kind == ItemKind::Shape
                && bar.page == row.page
                && b.height() < 1.2
                && b.width() < 4.0 * row.size
                && lb.x0 - 1.0 <= b.x0
                && b.x1 <= lb.x1 + 1.0
                && row.ink.y0 - 1.0 <= b.y0
                && b.y0 <= row.ink.y1 + 1.0
                && row.items.iter().any(|&k| {
                    let i = &items[k];
                    small(i) && i.bbox.y1 <= b.y0 + 0.5 && overlaps_x(&i.bbox, b)
                })
                && row.items.iter().any(|&k| {
                    let i = &items[k];
                    small(i) && i.bbox.y0 >= b.y1 - 0.5 && overlaps_x(&i.bbox, b)
                })
        });
        if has_fraction {
            fractions.push((row, cell));
        }
    }
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for &(upper, cell) in &fractions {
        let a = &upper.ink;
        let Some(&(lower, _)) = fractions
            .iter()
            .filter(|(r, c)| {
                r.page == upper.page
                    && *c != cell
                    && r.ink.y0 > a.y0 + 0.5 * upper.size
                    && overlaps_x(&r.ink, a)
            })
            .min_by(|x, y| x.0.ink.y0.total_cmp(&y.0.ink.y0))
        else {
            continue;
        };
        let b = &lower.ink;
        let size = upper.size.max(lower.size);
        let gap = b.y0 - a.y1;
        if gap >= MIN_FRACTION_ROW_GAP_EM * size {
            continue;
        }
        let divided = items.iter().any(|s| {
            s.kind == ItemKind::Shape
                && s.page == upper.page
                && s.bbox.height() >= 1.2
                && s.bbox.y0 >= a.y1 - 1.0
                && s.bbox.y1 <= b.y0 + 1.0
                && s.bbox.x0 < a.x1.min(b.x1)
                && s.bbox.x1 > a.x0.max(b.x0)
        });
        if divided || !seen.insert((upper.page, a.y1.round() as i64)) {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            CRAMPED_FRACTIONS,
            Severity::Warning,
            format!(
                "two rows of fractions on page {} are only {}pt apart, so their numerators \
                 and denominators crowd each other",
                upper.page + 1,
                round1(gap.max(0.0))
            ),
        );
        diagnostic.measured = Some(round1(gap));
        diagnostic.limit = Some(round1(MIN_FRACTION_ROW_GAP_EM * size));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "rows holding built-up fractions need more space than rows of text: raise the \
             grid's row-gutter (about 0.8 to 1em) or the cells' inset; the leading of \
             paragraphs with an inline fraction can stay as it is"
                .into(),
        );
        let line = &geometry.lines[lower.line];
        out.push(located(world, diagnostic, line_span(geometry, line), upper.page, a.union(b)));
    }
    out
}

// --- L086: loosely set display lines ----------------------------------------------

/// A title, name or headline of several lines set with the leading of running
/// text, so its lines fall apart.
fn loose_display_lines(
    world: &LabWorld,
    geometry: &Geometry,
    rows: &[Row],
    body: f64,
) -> Vec<Diagnostic> {
    let paragraph = |line: &Line| {
        ancestor(geometry, line.elem, |e| e.name == "par" || e.name == "heading")
    };
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for pair in rows.windows(2) {
        let (ra, rb) = (&pair[0], &pair[1]);
        if rb.line != ra.line + 1 || ra.page != rb.page {
            continue;
        }
        let (a, b) = (&geometry.lines[ra.line], &geometry.lines[rb.line]);
        let Some(par) = paragraph(a) else { continue };
        let size = ra.size;
        if paragraph(b) != Some(par)
            || (ra.size - rb.size).abs() > 0.5
            || size < DISPLAY_PT.max(DISPLAY_RATIO * body)
        {
            continue;
        }
        let pitch = (b.bbox.y0 + b.baseline) - (a.bbox.y0 + a.baseline);
        let clear = rb.ink.y0 - ra.ink.y1;
        if pitch <= MAX_DISPLAY_PITCH_EM * size
            || clear <= MAX_DISPLAY_CLEAR_EM * size
            || !seen.insert(par)
        {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            LOOSE_DISPLAY,
            Severity::Warning,
            format!(
                "the lines of {}pt display text \"{}\" on page {} are {} times its size \
                 apart, so they fall apart into separate lines",
                round1(size),
                short(&ra.text),
                ra.page + 1,
                round1(pitch / size)
            ),
        );
        diagnostic.measured = Some(round1(pitch / size));
        diagnostic.limit = Some(MAX_DISPLAY_PITCH_EM);
        diagnostic.unit = Some("ratio");
        diagnostic.hints.push(
            "set display text with its own tight leading, in its own size: a line pitch of \
             about 1.1 to 1.25 times the size (set par(leading: ..) inside the \
             text(size: ..) block that holds it)"
                .into(),
        );
        out.push(located(world, diagnostic, line_span(geometry, a), ra.page, ra.ink.union(&rb.ink)));
    }
    out
}

// --- L087: smaller text drifting away from its large line --------------------

/// A subtitle, standfirst or title under a large line (a name, a time, a
/// product name) set so far below it, and no closer to it than to what
/// follows, that it drifts away. Typically paragraph spacing in em, which is
/// measured in the large text's own size. L064 judges the document title.
fn drifting_subtitles(
    world: &LabWorld,
    geometry: &Geometry,
    body: f64,
    earlier: &[Diagnostic],
) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let all = rows(geometry, |line| 0.3 * line.bbox.height() + 1.0);
    let boxes: Vec<Option<usize>> =
        all.iter().map(|r| visible_box(geometry, items[r.items[0]].container)).collect();
    let judged: Vec<(usize, BBox)> = earlier
        .iter()
        .filter(|d| d.rule == crate::titles::DETACHED_SUBTITLE)
        .filter_map(|d| Some((d.page?.checked_sub(1)?, d.bbox?)))
        .collect();
    let mut out = Vec::new();
    for (index, row) in all.iter().enumerate() {
        let (a, size, page) = (&row.ink, row.size, row.page);
        if size < DISPLAY_PT.max(DISPLAY_RATIO * body) {
            continue;
        }
        let below: Vec<usize> = (0..all.len())
            .filter(|&k| {
                let r = &all[k];
                r.page == page
                    && r.ink.y0 > a.y1 - 0.2 * size
                    && r.ink.x0 < a.x1 + 2.0
                    && a.x0 < r.ink.x1 + 2.0
            })
            .collect();
        let Some(&next) = below.iter().min_by(|x, y| all[**x].ink.y0.total_cmp(&all[**y].ink.y0))
        else {
            continue;
        };
        let (n, nb) = (&all[next], &all[next].ink);
        if n.size >= size / 1.15 || n.size < body + 0.4 || boxes[next] != boxes[index] {
            continue;
        }
        // A side label: text beside it on its row, in the same box.
        let side = (0..all.len()).any(|k| {
            let r = &all[k].ink;
            all[k].page == page
                && boxes[k] == boxes[index]
                && r.x0 > a.x1 + 2.0
                && r.x0 - a.x1 < 3.0 * size
                && r.y0 < a.y1
                && a.y0 < r.y1
        });
        // Text set inside a drawn shape (a badge, an avatar).
        let badge = items.iter().any(|s| {
            s.kind == ItemKind::Shape
                && s.page == page
                && s.bbox.x0 <= a.x0
                && s.bbox.x1 >= a.x1
                && s.bbox.y0 <= a.y0
                && s.bbox.y1 >= a.y1
                && s.bbox.y1 < nb.y1
        });
        let aligned = (nb.x0 - a.x0).abs() <= 3.0
            || ((nb.x0 + nb.x1) - (a.x0 + a.x1)).abs() <= 6.0;
        let divided = items.iter().any(|i| {
            i.kind != ItemKind::Text
                && i.kind != ItemKind::Link
                && i.page == page
                && i.bbox.y0 >= a.y1 - 0.5
                && i.bbox.y1 <= nb.y0 + 0.5
                && i.bbox.x0 < a.x1.max(nb.x1)
                && i.bbox.x1 > a.x0.min(nb.x0)
        });
        let gap = nb.y0 - a.y1;
        if side || badge || !aligned || divided || gap <= MAX_SUBTITLE_GAP_EM * size {
            continue;
        }
        // The smaller text's own lines, and the space below them.
        let mut sub = *nb;
        let mut following = below.clone();
        following.sort_by(|x, y| all[*x].ink.y0.total_cmp(&all[*y].ink.y0));
        for k in following {
            let r = &all[k];
            if k != next
                && (r.size - n.size).abs() < 0.5
                && r.ink.y0 > sub.y0
                && r.ink.y0 - sub.y1 < 0.6 * n.size
            {
                sub.y1 = sub.y1.max(r.ink.y1);
            }
        }
        let after = items
            .iter()
            .filter(|i| {
                i.kind != ItemKind::Link
                    && i.page == page
                    && i.bbox.y0 > sub.y1 + 0.5
                    && overlaps_x(&i.bbox, &sub)
            })
            .map(|i| i.bbox.y0 - sub.y1)
            .fold(f64::INFINITY, f64::min);
        if gap < after {
            continue;
        }
        let bbox = a.union(nb);
        if judged.iter().any(|(p, b)| *p == page && b.intersect(&bbox).is_some()) {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            DRIFTING_SUBTITLE,
            Severity::Warning,
            format!(
                "the text below the {}pt line \"{}\" on page {} starts {}pt under it, {} \
                 times its size and no closer to it than to what follows, so it drifts \
                 away from the line it belongs to",
                round1(size),
                short(&row.text),
                page + 1,
                round1(gap),
                round1(gap / size)
            ),
        );
        diagnostic.measured = Some(round1(gap));
        diagnostic.limit = Some(round1(MAX_SUBTITLE_GAP_EM * size));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "set the smaller text about 0.3 to 0.6 times the large text's size below it \
             (both in one stack(spacing: ..) or block), not with paragraph spacing in em, \
             which is measured in the large text's own size"
                .into(),
        );
        let line = &geometry.lines[n.line];
        out.push(located(world, diagnostic, line_span(geometry, line), page, bbox));
    }
    out
}

// --- L088: a short label that wraps -------------------------------------------

/// A short label in the first column of a grid or table ("Maid of Honour",
/// "Frequency response") that wraps onto a second line while the other labels
/// of its column fit on one: the label column is too narrow.
fn wrapped_labels(world: &LabWorld, geometry: &Geometry, rows: &[Row]) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let grid_of = |elem| ancestor(geometry, elem, |e| e.name == "grid" || e.name == "table");
    // The text lines of each cell.
    let mut cells: BTreeMap<(usize, usize), Vec<&Row>> = BTreeMap::new();
    for row in rows {
        if let Some(cell) = ancestor(geometry, items[row.items[0]].elem, |e| e.name == "cell") {
            cells.entry((cell, row.page)).or_default().push(row);
        }
    }
    // Cells of a column share the left edge of their first line.
    let mut columns: BTreeMap<(usize, i64), Vec<Vec<&Row>>> = BTreeMap::new();
    for ((_, page), mut lines) in cells {
        lines.sort_by(|a, b| a.ink.y0.total_cmp(&b.ink.y0));
        let left = lines[0].ink.x0;
        columns.entry((page, (left / 2.0).round() as i64)).or_default().push(lines);
    }
    let mut out = Vec::new();
    for ((page, _), members) in columns {
        let single: Vec<&Vec<&Row>> = members.iter().filter(|m| m.len() == 1).collect();
        if members.len() < 3
            || single.iter().filter(|m| m[0].text.trim().chars().count() <= SHORT_LABEL_CHARS).count()
                < 2
        {
            continue;
        }
        for cell in &members {
            let [first, second] = cell.as_slice() else { continue };
            if (first.size - second.size).abs() > 0.3 {
                continue;
            }
            let full = format!("{} {}", first.text.trim_end(), second.text.trim_start());
            let full = full.trim();
            if full.split_whitespace().count() > SHORT_LABEL_WORDS
                || full.chars().count() > SHORT_LABEL_CHARS
            {
                continue;
            }
            // The first line ended because the next word did not fit.
            let rest = second.text.trim();
            let word = rest.split_whitespace().next().unwrap_or_default();
            let width: f64 = second.items.iter().map(|&k| items[k].bbox.width()).sum();
            let word_width =
                width * word.chars().count() as f64 / rest.chars().count().max(1) as f64;
            let line = &geometry.lines[first.line];
            if line.natural + 0.25 * first.size + word_width <= line.available {
                continue;
            }
            // The label column: nothing of the same grid to its left.
            let grid = grid_of(items[first.items[0]].elem);
            let lb = &line.bbox;
            let leftmost = !items.iter().any(|i| {
                nonblank(i)
                    && i.page == page
                    && grid_of(i.elem) == grid
                    && i.bbox.x1 <= lb.x0 - 1.0
                    && lb.x0 - i.bbox.x1 < 200.0
                    && i.bbox.y0 < lb.y1
                    && lb.y0 < i.bbox.y1
            });
            if !leftmost {
                continue;
            }
            let mut diagnostic = Diagnostic::new(
                WRAPPED_LABEL,
                Severity::Warning,
                format!(
                    "the label \"{full}\" on page {} wraps onto two lines while the other \
                     labels of its column fit on one",
                    page + 1
                ),
            );
            diagnostic.hints.push(
                "give the label column enough width for its longest label (an auto column \
                 or a wider fixed width), or shorten the label"
                    .into(),
            );
            out.push(located(
                world,
                diagnostic,
                line_span(geometry, line),
                page,
                first.ink.union(&second.ink),
            ));
        }
    }
    out
}
