use std::collections::{BTreeMap, BTreeSet};

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, Line, round1};
use crate::structure::enclosing_cell;
use crate::world::LabWorld;

pub const CENTERED_PARAGRAPH: &str = "L044.centered-paragraph";
pub const FIGURE_SPACING: &str = "L045.figure-too-close";
pub const LOPSIDED_COLUMNS: &str = "L046.lopsided-columns";

/// A cell wrapping into this many lines is cramped when a neighbour has room.
const CRAMPED_CELL_LINES: usize = 3;

/// A single-line neighbour filled less than this, with at least
/// `MIN_SLACK_PT` to spare, could give width away.
const MAX_ROOMY_FILL: f64 = 0.6;
const MIN_SLACK_PT: f64 = 40.0;

/// Rows that must show the pattern before the table is reported.
const MIN_LOPSIDED_ROWS: usize = 2;

/// A measure at least this share of the text area is wide; centred text that
/// wraps across it is hard to read. Convention.
const WIDE_MEASURE_SHARE: f64 = 0.6;

/// A line this full was broken by wrapping, not by a deliberate line break.
const WRAPPED_SHARE: f64 = 0.85;

/// A centred line leaves at least this share of the measure on each side.
const CENTERED_SIDE_SHARE: f64 = 0.1;

/// Display text (titles) may be centred over several lines.
const DISPLAY_TEXT_PT: f64 = 14.0;

/// A figure or table must be set off from the surrounding text by at least
/// a blank line: this many line pitches more than the text's own line
/// spacing. Less, and it reads as part of the paragraph.
const MIN_FIGURE_EXTRA_LINES: f64 = 1.0;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = centered_paragraphs(world, geometry);
    out.extend(figure_spacing(world, geometry));
    out.extend(lopsided_columns(world, geometry));
    out.extend(title_proximity(world, geometry));
    out.extend(paragraph_spacing(world, geometry));
    out.extend(unseparated_terms(world, geometry));
    out.extend(sparse_bars(world, geometry));
    out.extend(chart_labels(world, geometry));
    out
}

pub const CHART_LABELS: &str = "L063.chart-labels";

/// Category labels under a bar chart further below the bars than this many
/// times their size are detached from the axis.
const MAX_LABEL_GAP_EM: f64 = 2.5;

/// A category label whose centre is further from its bar's centre than this
/// share of the bar width sits under the wrong place.
const MAX_LABEL_OFFSET: f64 = 0.3;

/// Bar charts whose category labels are far below the bars or not centred
/// under them.
fn chart_labels(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut rows: BTreeMap<(usize, i64), Vec<&crate::geom::Item>> = BTreeMap::new();
    for shape in geometry.items.iter().filter(|i| i.kind == ItemKind::Shape) {
        let b = &shape.bbox;
        if shape.color.is_some() && b.height() >= 4.0 && b.width() >= 2.0 {
            rows.entry((shape.page, (b.y1 * 2.0).round() as i64))
                .or_default()
                .push(shape);
        }
    }
    let texts: Vec<&crate::geom::Item> = geometry
        .items
        .iter()
        .filter(|i| {
            i.kind == ItemKind::Text
                && i.font_size.is_some()
                && i.text.as_deref().is_some_and(|t| !t.trim().is_empty())
        })
        .collect();
    let centre = |b: &BBox| (b.x0 + b.x1) / 2.0;
    let mut out = Vec::new();
    for ((page, bottom), mut bars) in rows {
        if bars.len() < 3 {
            continue;
        }
        bars.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
        let x0 = bars[0].bbox.x0;
        let x1 = bars.iter().map(|b| b.bbox.x1).fold(f64::NEG_INFINITY, f64::max);
        let heights = bars.iter().map(|b| b.bbox.height());
        let spread = heights.clone().fold(f64::NEG_INFINITY, f64::max)
            - heights.fold(f64::INFINITY, f64::min);
        // Bars of one height are table cells or swatches, not a chart.
        if spread < 1.0 || bars.iter().any(|b| b.bbox.width() > 0.25 * (x1 - x0)) {
            continue;
        }
        let base = bottom as f64 / 2.0;
        let below: Vec<&&crate::geom::Item> = texts
            .iter()
            .filter(|t| {
                t.page == page
                    && t.bbox.y0 > base - 1.0
                    && t.bbox.y0 < base + 120.0
                    && (x0 - 20.0..=x1 + 20.0).contains(&centre(&t.bbox))
            })
            .collect();
        let Some(top) = below.iter().map(|t| t.bbox.y0).min_by(f64::total_cmp) else {
            continue;
        };
        let mut labels: Vec<&&crate::geom::Item> =
            below.into_iter().filter(|t| (t.bbox.y0 - top).abs() < 1.5).collect();
        if labels.len() < 3 {
            continue;
        }
        labels.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
        let size = labels.iter().filter_map(|t| t.font_size).fold(0.0, f64::max);
        let gap = top - base;
        let first = labels[0];
        let mut report = |message: String, measured: f64, limit: f64, hint: &str| {
            let mut diagnostic =
                Diagnostic::new(CHART_LABELS, Severity::Warning, message);
            diagnostic.loc =
                diag::resolve_span(world, geometry.span_for(first.span, first.elem));
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = Some(BBox { x0, y0: base, x1, y1: top + size }.rounded());
            diagnostic.measured = Some(round1(measured));
            diagnostic.limit = Some(round1(limit));
            diagnostic.hints.push(hint.into());
            out.push(diagnostic);
        };
        if gap > MAX_LABEL_GAP_EM * size {
            report(
                format!(
                    "the category labels of a chart on page {} sit {}pt below the bars, \
                     {}x their size, detached from the axis",
                    page + 1,
                    round1(gap),
                    round1(gap / size)
                ),
                gap,
                MAX_LABEL_GAP_EM * size,
                "place the labels directly under the axis (a few points below the bars) \
                 instead of in a separate row further down",
            );
        }
        if labels.len() == bars.len() {
            let width =
                bars.iter().map(|b| b.bbox.width()).sum::<f64>() / bars.len() as f64;
            let offset = labels
                .iter()
                .zip(&bars)
                .map(|(t, b)| (centre(&t.bbox) - centre(&b.bbox)).abs())
                .fold(0.0, f64::max);
            if offset > MAX_LABEL_OFFSET * width {
                report(
                    format!(
                        "the category labels of a chart on page {} are not centred under \
                         their bars: one is {}pt off, {}x the bar width",
                        page + 1,
                        round1(offset),
                        round1(offset / width)
                    ),
                    offset / width,
                    MAX_LABEL_OFFSET,
                    "lay out labels and bars in the same grid columns (or compute both \
                     positions from the same values) so each label centres under its bar",
                );
            }
        }
    }
    out
}

pub const UNSEPARATED_TERMS: &str = "L056.unseparated-terms";
pub const SPARSE_BARS: &str = "L057.sparse-bar-chart";

/// Entries of an unmarked term list need this much more space between them
/// than between their own lines once they wrap.
const MIN_ENTRY_GAP_EM: f64 = 0.25;

/// The gap between bar groups of a chart, relative to a group's width, above
/// which the bars float apart. Charting convention keeps it below about 1.
const MAX_BAR_GAP_RATIO: f64 = 1.5;

/// The term list entry (a direct child of a terms element) holding content.
fn term_entry(geometry: &Geometry, mut cursor: Option<usize>) -> Option<(usize, usize)> {
    while let Some(index) = cursor {
        let parent = geometry.elements[index].parent;
        if let Some(list) = parent
            && geometry.elements[list].name == "terms"
        {
            return Some((list, index));
        }
        cursor = parent;
    }
    None
}

/// Term lists whose entries wrap but are set as close as their own lines, so
/// that, without markers, one entry runs into the next.
fn unseparated_terms(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut lists: BTreeMap<usize, Vec<(usize, &Line)>> = BTreeMap::new();
    for line in &geometry.lines {
        if let Some((list, entry)) = term_entry(geometry, line.elem) {
            lists.entry(list).or_default().push((entry, line));
        }
    }
    let mut out = Vec::new();
    for (list, lines) in lists {
        let mut within = Vec::new();
        let mut between = Vec::new();
        for pair in lines.windows(2) {
            let ((ea, a), (eb, b)) = (pair[0], pair[1]);
            let distance = baseline(b) - baseline(a);
            if a.page != b.page || distance <= 0.0 {
                continue;
            }
            if ea == eb { within.push(distance) } else { between.push(distance) }
        }
        if within.len() < 2 || between.len() < 2 {
            continue;
        }
        within.sort_by(f64::total_cmp);
        between.sort_by(f64::total_cmp);
        let (pitch, gap) = (within[within.len() / 2], between[between.len() / 2]);
        let Some(size) = crate::spacing::line_size(geometry, lines[0].1) else {
            continue;
        };
        if gap - pitch >= MIN_ENTRY_GAP_EM * size {
            continue;
        }
        let first = lines[0].1;
        let mut diagnostic = Diagnostic::new(
            UNSEPARATED_TERMS,
            Severity::Warning,
            format!(
                "the entries of a term list on page {} wrap but are set {}pt apart, as \
                 close as their own lines ({}pt), so they run together",
                first.page + 1,
                round1(gap),
                round1(pitch)
            ),
        );
        diagnostic.loc = diag::resolve_span(
            world,
            geometry.span_for(geometry.elements[list].span, Some(list)),
        );
        diagnostic.page = Some(first.page + 1);
        diagnostic.measured = Some(round1(gap - pitch));
        diagnostic.limit = Some(round1(MIN_ENTRY_GAP_EM * size));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "separate the entries: set terms(tight: false) or terms(spacing: 0.8em), or \
             set each description below its term"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Bar charts whose bars stand far apart on their axis: the gap between bar
/// groups is much wider than the groups themselves.
fn sparse_bars(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let shapes: Vec<&crate::geom::Item> =
        geometry.items.iter().filter(|i| i.kind == ItemKind::Shape).collect();
    let mut rows: BTreeMap<(usize, i64), Vec<&crate::geom::Item>> = BTreeMap::new();
    for shape in &shapes {
        let b = &shape.bbox;
        if shape.color.is_some() && b.height() >= 4.0 && b.width() >= 2.0 {
            rows.entry((shape.page, (b.y1 * 2.0).round() as i64))
                .or_default()
                .push(shape);
        }
    }
    let mut out = Vec::new();
    for ((page, bottom), mut bars) in rows {
        if bars.len() < 3 {
            continue;
        }
        bars.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
        let x0 = bars[0].bbox.x0;
        let x1 = bars.iter().map(|b| b.bbox.x1).fold(f64::NEG_INFINITY, f64::max);
        if bars.iter().any(|b| b.bbox.width() > 0.25 * (x1 - x0)) {
            continue;
        }
        // Only bars standing on an axis line form a chart.
        let y = bottom as f64 / 2.0;
        let on_axis = shapes.iter().any(|a| {
            a.page == page
                && a.bbox.height() < 1.5
                && ((a.bbox.y0 + a.bbox.y1) / 2.0 - y).abs() < 2.0
                && a.bbox.x0 <= x0 + 1.0
                && a.bbox.x1 >= x1 - 1.0
        });
        if !on_axis {
            continue;
        }
        let mut groups: Vec<BBox> = vec![bars[0].bbox];
        for pair in bars.windows(2) {
            let (a, b) = (&pair[0].bbox, &pair[1].bbox);
            let gap = b.x0 - a.x1;
            if gap < 0.25 * (a.width() + b.width()) {
                let last = groups.last_mut().unwrap();
                *last = last.union(b);
            } else {
                groups.push(*b);
            }
        }
        if groups.len() < 2 {
            continue;
        }
        let width = groups.iter().map(BBox::width).sum::<f64>() / groups.len() as f64;
        let gap = groups.windows(2).map(|g| g[1].x0 - g[0].x1).fold(0.0, f64::max);
        let ratio = gap / width;
        if ratio <= MAX_BAR_GAP_RATIO {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            SPARSE_BARS,
            Severity::Warning,
            format!(
                "the bars of a chart on page {} stand {}pt apart, {}x the width of a bar \
                 group ({}pt), so the chart looks empty",
                page + 1,
                round1(gap),
                round1(ratio),
                round1(width)
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(bars[0].span, bars[0].elem));
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = Some(BBox { x0, x1, ..groups[0] }.rounded());
        diagnostic.measured = Some(round1(ratio));
        diagnostic.limit = Some(MAX_BAR_GAP_RATIO);
        diagnostic.hints.push(
            "widen the bars or narrow the chart so that the gaps between groups are no \
             wider than the groups, especially with few categories"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

pub const TITLE_PROXIMITY: &str = "L054.title-too-close";
pub const PARAGRAPH_SPACING: &str = "L055.paragraph-spacing";

/// Text at least this large is display text: titles and headlines.
const DISPLAY_PT: f64 = 14.0;

/// The text below a multi-line title needs at least this many times the gap
/// between the title's own lines. Measured on 125 generated documents with
/// the text's boxes: titles read as separate from 2 times on; at 1.2 and 1.47
/// the text below reads as another title line.
const MIN_TITLE_AFTER_RATIO: f64 = 1.5;

/// Paragraphs without a first-line indent need at least this much more space
/// between them than between their lines (Typst's default is about 0.55em).
const MIN_PARAGRAPH_GAP_EM: f64 = 0.5;

/// Body text sizes for which paragraph spacing is judged.
const BODY_PT: std::ops::RangeInclusive<f64> = 8.0..=13.0;

fn baseline(line: &Line) -> f64 {
    line.bbox.y0 + line.baseline
}

/// Text that follows a multi-line title or headline closer than the title's
/// own lines are to each other, so it reads as part of the title.
fn title_proximity(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let lines = &geometry.lines;
    let sizes: Vec<Option<f64>> =
        lines.iter().map(|l| crate::spacing::line_size(geometry, l)).collect();
    let mut out = Vec::new();
    for k in 2..lines.len() {
        let (first, last, next) = (&lines[k - 2], &lines[k - 1], &lines[k]);
        let (Some(a), Some(size), Some(next_size)) =
            (sizes[k - 2], sizes[k - 1], sizes[k])
        else {
            continue;
        };
        if first.page != last.page
            || last.page != next.page
            || size < DISPLAY_PT
            || (a - size).abs() > 0.1
            || next_size >= size / 1.15
            || next.bbox.x0 >= last.bbox.x1
            || last.bbox.x0 >= next.bbox.x1
        {
            continue;
        }
        let within = baseline(last) - baseline(first);
        let below = baseline(next) - baseline(last);
        if within <= 0.0 || below <= 0.0 || within > 2.0 * size {
            continue;
        }
        // Clear space between the title's lines, and between its last line
        // and the next text, measured on the text's own boxes (display faces
        // differ widely in ascent and descent).
        let (Some(top), Some(bottom), Some(below_ink)) =
            (ink(geometry, first), ink(geometry, last), ink(geometry, next))
        else {
            continue;
        };
        let inside = bottom.y0 - top.y1;
        let after = below_ink.y0 - bottom.y1;
        if inside <= 0.0 || after >= MIN_TITLE_AFTER_RATIO * inside - 0.5 {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            TITLE_PROXIMITY,
            Severity::Warning,
            format!(
                "the text below a {}pt title on page {} starts {}pt under it, less than \
                 1.5 times the gap between the title's own lines ({}pt), so it reads as \
                 part of the title",
                round1(size),
                next.page + 1,
                round1(after.max(0.0)),
                round1(inside)
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(next.span, next.elem));
        diagnostic.page = Some(next.page + 1);
        diagnostic.bbox = Some(last.bbox.union(&next.bbox).rounded());
        diagnostic.measured = Some(round1(after));
        diagnostic.limit = Some(round1(MIN_TITLE_AFTER_RATIO * inside));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "leave clearly more space below the title than between its lines, e.g. \
             block(below: 0.8em) around the title or v(..) before the text that follows"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Body paragraphs set so close that they run together: barely more space
/// between paragraphs than between lines, and no first-line indent.
fn paragraph_spacing(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let lines = &geometry.lines;
    let excluded = |cursor| {
        ["cell", "list", "enum", "terms", "entry", "figure", "quote", "heading"]
            .iter()
            .any(|name| within_named(geometry, cursor, name))
    };
    let mut pitches = Vec::new();
    let mut gaps: Vec<(f64, f64, &Line)> = Vec::new();
    for pair in lines.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if a.page != b.page
            || a.container != b.container
            || (a.bbox.x0 - b.bbox.x0).abs() > 0.5
            || !geometry.is_body(a.region)
            || excluded(a.elem)
            || excluded(b.elem)
        {
            continue;
        }
        let (Some(pa), Some(pb)) = (
            crate::typography::paragraph_of(geometry, a.elem),
            crate::typography::paragraph_of(geometry, b.elem),
        ) else {
            continue;
        };
        let (Some(sa), Some(sb)) = (
            crate::spacing::line_size(geometry, a),
            crate::spacing::line_size(geometry, b),
        ) else {
            continue;
        };
        if (sa - sb).abs() > 0.1 || !BODY_PT.contains(&sa) {
            continue;
        }
        let distance = baseline(b) - baseline(a);
        if distance <= 0.0 {
            continue;
        }
        if pa == pb {
            pitches.push(distance);
        } else {
            gaps.push((distance, sa, b));
        }
    }
    if pitches.len() < 5 || gaps.is_empty() {
        return Vec::new();
    }
    let median = |values: &mut Vec<f64>| {
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    let pitch = median(&mut pitches);
    let mut extras: Vec<f64> = gaps.iter().map(|(d, ..)| d - pitch).collect();
    let extra = median(&mut extras);
    let (_, size, line) = gaps[0];
    let limit = MIN_PARAGRAPH_GAP_EM * size;
    if extra >= limit {
        return Vec::new();
    }
    let mut diagnostic = Diagnostic::new(
        PARAGRAPH_SPACING,
        Severity::Warning,
        format!(
            "paragraphs are set only {}pt further apart than their lines ({}pt), so they \
             run together; without a first-line indent they need at least {}pt",
            round1(extra.max(0.0)),
            round1(pitch),
            round1(limit)
        ),
    );
    diagnostic.loc = diag::resolve_span(world, geometry.span_for(line.span, line.elem));
    diagnostic.page = Some(line.page + 1);
    diagnostic.measured = Some(round1(extra));
    diagnostic.limit = Some(round1(limit));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "set par(spacing: ..) at least 0.5em larger than par(leading: ..) (Typst's \
         default is spacing 1.2em with leading 0.65em), or indent first lines; to meet a \
         page count, tighten the wording instead of the spacing"
            .into(),
    );
    vec![diagnostic]
}

/// A table cell and its lines.
type CellLines<'a> = (usize, &'a [&'a Line]);

/// How often a cramped column sat next to a roomy one.
struct Lopsided {
    rows: usize,
    most: usize,
    cell: usize,
    fill: f64,
}

/// Tables whose column widths are badly distributed: one column wraps every
/// cell into many lines while a neighbouring column stays on one line and is
/// mostly empty.
fn lopsided_columns(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut per_cell: BTreeMap<usize, Vec<&Line>> = BTreeMap::new();
    for line in &geometry.lines {
        if line.container.is_some_and(|c| geometry.containers[c].kind == "box") {
            continue;
        }
        if let Some(cell) = enclosing_cell(geometry, line.container) {
            per_cell.entry(cell).or_default().push(line);
        }
    }

    let mut rows: BTreeMap<(usize, i64), Vec<CellLines>> = BTreeMap::new();
    for (cell, lines) in &per_cell {
        let container = &geometry.containers[*cell];
        rows.entry((container.page, (container.bbox.y0 * 2.0).round() as i64))
            .or_default()
            .push((*cell, lines.as_slice()));
    }

    // Keyed by page and the left edges of the cramped and the roomy column.
    let mut pairs: BTreeMap<(usize, i64, i64), Lopsided> = BTreeMap::new();
    for ((page, _), cells) in &rows {
        for (cramped, cramped_lines) in cells {
            if cramped_lines.len() < CRAMPED_CELL_LINES {
                continue;
            }
            for (roomy, roomy_lines) in cells {
                let [line] = roomy_lines else { continue };
                if roomy == cramped || line.available <= 0.0 {
                    continue;
                }
                let fill = line.natural / line.available;
                if fill >= MAX_ROOMY_FILL || line.available - line.natural < MIN_SLACK_PT
                {
                    continue;
                }
                let key = (
                    *page,
                    geometry.containers[*cramped].bbox.x0.round() as i64,
                    geometry.containers[*roomy].bbox.x0.round() as i64,
                );
                let entry = pairs.entry(key).or_insert(Lopsided {
                    rows: 0,
                    most: 0,
                    cell: *cramped,
                    fill,
                });
                entry.rows += 1;
                entry.most = entry.most.max(cramped_lines.len());
                entry.fill = entry.fill.max(fill);
            }
        }
    }

    pairs
        .into_iter()
        .filter(|(_, found)| found.rows >= MIN_LOPSIDED_ROWS)
        .map(|((page, _, _), Lopsided { rows, most, cell, fill })| {
            let container = &geometry.containers[cell];
            let mut diagnostic = Diagnostic::new(
                LOPSIDED_COLUMNS,
                Severity::Warning,
                format!(
                    "a table on page {} wraps one column into up to {most} lines in {rows} \
                     rows while a neighbouring column stays on one line and is at most {}% \
                     full",
                    page + 1,
                    (fill * 100.0).round()
                ),
            );
            diagnostic.loc = diag::resolve_span(world, container.span);
            diagnostic.elem = Some("cell".into());
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = Some(container.bbox.rounded());
            diagnostic.measured = Some(most as f64);
            diagnostic.limit = Some(CRAMPED_CELL_LINES as f64);
            diagnostic.unit = Some("lines");
            diagnostic.hints.push(
                "rebalance the column widths: give the wrapping column a larger share (for \
                 example 2fr) and let short columns size to their content (auto)"
                    .into(),
            );
            diagnostic
        })
        .collect()
}

struct Ink {
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    size: f64,
}

/// The extent and largest size of the text on a line.
fn ink(geometry: &Geometry, line: &Line) -> Option<Ink> {
    let b = &line.bbox;
    let mut out: Option<Ink> = None;
    for item in &geometry.items {
        if item.page != line.page || item.kind != ItemKind::Text {
            continue;
        }
        let Some(size) = item.font_size else { continue };
        let cx = (item.bbox.x0 + item.bbox.x1) / 2.0;
        let cy = (item.bbox.y0 + item.bbox.y1) / 2.0;
        if cy < b.y0 || cy > b.y1 || cx < b.x0 - 1.0 || cx > b.x1 + 1.0 {
            continue;
        }
        out = Some(match out {
            Some(ink) => Ink {
                x0: ink.x0.min(item.bbox.x0),
                x1: ink.x1.max(item.bbox.x1),
                y0: ink.y0.min(item.bbox.y0),
                y1: ink.y1.max(item.bbox.y1),
                size: ink.size.max(size),
            },
            None => Ink {
                x0: item.bbox.x0,
                x1: item.bbox.x1,
                y0: item.bbox.y0,
                y1: item.bbox.y1,
                size,
            },
        });
    }
    out
}

fn body_width(geometry: &Geometry, page: usize) -> Option<f64> {
    geometry
        .regions
        .iter()
        .find(|r| r.kind == "page-body" && r.page == page)
        .map(|r| r.bbox.width())
}

fn centered_paragraphs(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for pair in geometry.lines.windows(2) {
        let (above, line) = (&pair[0], &pair[1]);
        if above.page != line.page
            || above.container != line.container
            || !geometry.is_body(line.region)
            || (above.bbox.x0 - line.bbox.x0).abs() > 0.5
            || (above.bbox.x1 - line.bbox.x1).abs() > 0.5
        {
            continue;
        }
        let step = line.bbox.y1 - above.bbox.y1;
        if step <= 0.0 || step > 2.5 * above.bbox.height() {
            continue;
        }
        let Some(width) = body_width(geometry, line.page) else { continue };
        if above.available < WIDE_MEASURE_SHARE * width
            || above.natural < WRAPPED_SHARE * above.available
        {
            continue;
        }
        let Some(ink) = ink(geometry, line) else { continue };
        if ink.size >= DISPLAY_TEXT_PT {
            continue;
        }
        let left = ink.x0 - line.bbox.x0;
        let right = line.bbox.x1 - ink.x1;
        let side = CENTERED_SIDE_SHARE * line.available;
        if left <= side
            || right <= side
            || (left - right).abs() >= (0.03 * line.available).max(2.0)
        {
            continue;
        }
        if !seen.insert((line.page, line.container)) {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            CENTERED_PARAGRAPH,
            Severity::Warning,
            format!(
                "a paragraph on page {} wraps across a {}pt measure and is centred, so \
                 its lines start at different places",
                line.page + 1,
                round1(line.available)
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(line.span, line.elem));
        diagnostic.elem = line.elem.map(|e| geometry.elements[e].name.into());
        diagnostic.page = Some(line.page + 1);
        diagnostic.bbox = Some(above.bbox.union(&line.bbox).rounded());
        diagnostic.hints.push(
            "align running text left or justify it (set align(left) or par(justify: \
             true) inside the block); centre only single lines such as titles and short \
             captions"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn within(geometry: &Geometry, mut cursor: Option<usize>, target: usize) -> bool {
    while let Some(index) = cursor {
        if index == target {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

fn within_named(geometry: &Geometry, mut cursor: Option<usize>, name: &str) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].name == name {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// The distance between this line's baseline and the nearest other line of
/// the same text block.
fn pitch(geometry: &Geometry, line: &Line) -> Option<f64> {
    geometry
        .lines
        .iter()
        .filter(|l| {
            l.page == line.page
                && l.container == line.container
                && (l.bbox.x0 - line.bbox.x0).abs() < 0.5
        })
        .map(|l| (l.bbox.y1 - line.bbox.y1).abs())
        .filter(|d| *d > 1.0)
        .min_by(f64::total_cmp)
}

fn figure_spacing(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, figure) in geometry.elements.iter().enumerate() {
        if figure.name != "figure" {
            continue;
        }
        for page_box in &figure.boxes {
            let (page, frame) = (page_box.page, page_box.bbox);
            let mut inner: Vec<&Line> = geometry
                .lines
                .iter()
                .filter(|l| l.page == page && within(geometry, l.elem, index))
                .collect();
            inner.sort_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1));
            let outer: Vec<&Line> = geometry
                .lines
                .iter()
                .filter(|l| {
                    l.page == page
                        && geometry.is_body(l.region)
                        && !within(geometry, l.elem, index)
                        && !within_named(geometry, l.elem, "heading")
                        && !within_named(geometry, l.elem, "figure")
                        && l.bbox.x0 < frame.x1
                        && frame.x0 < l.bbox.x1
                })
                .collect();

            let before = outer
                .iter()
                .filter(|l| l.bbox.y1 <= frame.y0 + 1.0)
                .max_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1));
            if let Some(line) = before {
                let size = ink(geometry, line).map_or(10.0, |i| i.size);
                // A single-line paragraph has no pitch of its own; the nearest
                // line of its block may lie across the figure.
                let pitch = pitch(geometry, line)
                    .filter(|p| *p <= 2.0 * size)
                    .unwrap_or(1.3 * size);
                let extra =
                    match inner.first().filter(|f| f.bbox.y0 <= frame.y0 + 0.5 * size) {
                        Some(first) => first.bbox.y1 - line.bbox.y1 - pitch,
                        None => frame.y0 - line.bbox.y1 - 0.25 * size,
                    };
                let limit = MIN_FIGURE_EXTRA_LINES * pitch;
                if extra < limit && extra > -size {
                    out.push(spacing_diagnostic(
                        world, geometry, index, page, "above", extra, limit,
                    ));
                }
            }

            let after = outer
                .iter()
                .filter(|l| l.bbox.y0 >= frame.y1 - 1.0)
                .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
            if let Some(line) = after {
                let size = ink(geometry, line).map_or(10.0, |i| i.size);
                // A single-line paragraph has no pitch of its own; the nearest
                // line of its block may lie across the figure.
                let pitch = pitch(geometry, line)
                    .filter(|p| *p <= 2.0 * size)
                    .unwrap_or(1.3 * size);
                let extra =
                    match inner.last().filter(|l| l.bbox.y1 >= frame.y1 - 0.5 * size) {
                        Some(last) => line.bbox.y1 - last.bbox.y1 - pitch,
                        None => line.bbox.y0 - frame.y1,
                    };
                let limit = MIN_FIGURE_EXTRA_LINES * pitch;
                if extra < limit && extra > -size {
                    out.push(spacing_diagnostic(
                        world, geometry, index, page, "below", extra, limit,
                    ));
                }
            }
        }
    }
    out
}

fn spacing_diagnostic(
    world: &LabWorld,
    geometry: &Geometry,
    figure: usize,
    page: usize,
    side: &'static str,
    extra: f64,
    limit: f64,
) -> Diagnostic {
    let element = &geometry.elements[figure];
    let mut diagnostic = Diagnostic::new(
        FIGURE_SPACING,
        Severity::Warning,
        format!(
            "a figure or table on page {} is set off from the text {side} it by only {}pt \
             more than the line spacing; it needs about a blank line ({}pt) or it reads \
             as part of the paragraph",
            page + 1,
            round1(extra.max(0.0)),
            round1(limit)
        ),
    );
    diagnostic.loc = diag::resolve_span(world, element.span);
    diagnostic.elem = Some("figure".into());
    diagnostic.label = element.label.clone();
    diagnostic.page = Some(page + 1);
    diagnostic.measured = Some(round1(extra));
    diagnostic.limit = Some(round1(limit));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "give figures and tables about a blank line above and below (show figure: set \
         block(above: 2em, below: 2em)) instead of reducing it to save space"
            .into(),
    );
    diagnostic
}
