use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, Line, round1};
use crate::world::LabWorld;

pub const COLUMN_IMBALANCE: &str = "L019.column-imbalance";
pub const LINE_LENGTH: &str = "L020.line-length";
pub const EMPTY_PAGE: &str = "L021.empty-page";

/// How many lines the bottoms of side-by-side columns may differ.
const MAX_COLUMN_DIFF_LINES: f64 = 3.0;

/// Comfortable characters per line of body text. Convention.
const MIN_CHARS_PER_LINE: f64 = 30.0;
const MAX_CHARS_PER_LINE: f64 = 115.0;

/// Full body lines needed before line length is judged.
const MIN_MEASURED_LINES: usize = 5;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = empty_pages(geometry);
    out.extend(column_imbalance(world, geometry));
    out.extend(line_length(world, geometry));
    out
}

fn empty_pages(geometry: &Geometry) -> Vec<Diagnostic> {
    if geometry.pages.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (index, region) in geometry.regions.iter().enumerate() {
        if region.kind != "page-body" {
            continue;
        }
        let has_content = geometry.items.iter().any(|item| {
            item.region == Some(index)
                && item.kind != ItemKind::Link
                && item.visible().is_some()
        });
        if has_content {
            continue;
        }
        let page = region.page + 1;
        let mut diagnostic = Diagnostic::new(
            EMPTY_PAGE,
            Severity::Warning,
            format!("page {page} has no content in its text area"),
        );
        diagnostic.page = Some(page);
        diagnostic.bbox = Some(region.bbox.rounded());
        diagnostic.hints.push(
            "remove consecutive page breaks, or use pagebreak(to: ..) only where a blank \
             page is intended"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Lines of body paragraphs, grouped into side-by-side columns per page.
fn columns(geometry: &Geometry) -> BTreeMap<usize, Vec<Vec<&Line>>> {
    let mut pages: BTreeMap<usize, BTreeMap<(i64, i64), Vec<&Line>>> = BTreeMap::new();
    for line in &geometry.lines {
        if !is_body_text(geometry, line) || !in_body(geometry, line) {
            continue;
        }
        let key = (line.bbox.x0.round() as i64, line.bbox.width().round() as i64);
        pages.entry(line.page).or_default().entry(key).or_default().push(line);
    }
    pages
        .into_iter()
        .map(|(page, groups)| (page, groups.into_values().collect()))
        .collect()
}

/// Whether a line is running body text: part of a paragraph, or inline text
/// that Typst does not wrap in a paragraph element (e.g. a container without
/// paragraph breaks).
fn is_body_text(geometry: &Geometry, line: &Line) -> bool {
    line.elem.is_none_or(|e| geometry.elements[e].name == "par")
}

fn in_body(geometry: &Geometry, line: &Line) -> bool {
    geometry.regions.iter().any(|r| {
        r.page == line.page
            && r.kind == "page-body"
            && r.bbox.intersect(&line.bbox).is_some()
    })
}

fn pitch(lines: &[&Line]) -> Option<f64> {
    let mut steps: Vec<f64> = lines
        .windows(2)
        .map(|w| w[1].bbox.y0 - w[0].bbox.y0)
        .filter(|d| *d > 0.0)
        .collect();
    steps.sort_by(f64::total_cmp);
    steps.get(steps.len() / 2).copied()
}

fn column_imbalance(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (page, groups) in columns(geometry) {
        if page + 1 == geometry.pages.len() {
            continue;
        }
        let mut side_by_side: Vec<&Vec<&Line>> = groups
            .iter()
            .filter(|group| group.len() >= 2)
            .filter(|group| {
                groups.iter().any(|other| {
                    !std::ptr::eq(*group, other)
                        && other.len() >= 2
                        && disjoint_x(&span(group), &span(other))
                        && overlap_y(&span(group), &span(other))
                })
            })
            .collect();
        if side_by_side.len() < 2 {
            continue;
        }
        side_by_side.sort_by(|a, b| span(a).x0.total_cmp(&span(b).x0));
        let Some(pitch) = pitch(side_by_side[0]) else { continue };
        let bottoms: Vec<f64> = side_by_side.iter().map(|g| span(g).y1).collect();
        let highest = bottoms.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let last = *bottoms.last().unwrap();
        let diff = highest - last;
        if diff <= MAX_COLUMN_DIFF_LINES * pitch {
            continue;
        }
        let shortest = side_by_side.last().unwrap();
        let mut diagnostic = Diagnostic::new(
            COLUMN_IMBALANCE,
            Severity::Warning,
            format!(
                "the last column on page {} ends {}pt ({:.0} lines) above the others",
                page + 1,
                round1(diff),
                diff / pitch
            ),
        );
        diagnostic.anchor = Some(shortest[0].span);
        diagnostic.loc = diag::resolve_span(world, shortest[0].span);
        diagnostic.page = Some(page + 1);
        diagnostic.bbox =
            Some(BBox { y0: last, y1: highest, ..span(shortest) }.rounded());
        diagnostic.measured = Some(round1(diff));
        diagnostic.limit = Some(round1(MAX_COLUMN_DIFF_LINES * pitch));
        diagnostic.unit = Some("pt");
        diagnostic
            .hints
            .push("balance the columns (columns(.., balanced: true))".into());
        out.push(diagnostic);
    }
    out
}

fn span(lines: &[&Line]) -> BBox {
    lines.iter().skip(1).fold(lines[0].bbox, |acc, l| acc.union(&l.bbox))
}

fn disjoint_x(a: &BBox, b: &BBox) -> bool {
    a.x1 <= b.x0 || b.x1 <= a.x0
}

fn overlap_y(a: &BBox, b: &BBox) -> bool {
    a.y0 < b.y1 && b.y0 < a.y1
}

fn line_length(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut items_by_page: BTreeMap<usize, Vec<(&BBox, usize)>> = BTreeMap::new();
    for item in &geometry.items {
        if let (ItemKind::Text, Some(text)) = (item.kind, &item.text) {
            items_by_page
                .entry(item.page)
                .or_default()
                .push((&item.bbox, text.chars().count()));
        }
    }

    let mut counts = Vec::new();
    let mut first = None;
    let lines = &geometry.lines;
    for (i, line) in lines.iter().enumerate() {
        let Some(elem) = line.elem else { continue };
        let is_last_of_par = lines.get(i + 1).is_none_or(|next| next.elem != Some(elem));
        if is_last_of_par
            || !is_body_text(geometry, line)
            || !line.justify
            || !in_body(geometry, line)
        {
            continue;
        }
        let chars: usize = items_by_page
            .get(&line.page)
            .into_iter()
            .flatten()
            .filter(|(bbox, _)| {
                let center = (bbox.y0 + bbox.y1) / 2.0;
                line.bbox.y0 <= center
                    && center <= line.bbox.y1
                    && bbox.x0 >= line.bbox.x0 - 1.0
                    && bbox.x1 <= line.bbox.x1 + 1.0
            })
            .map(|(_, count)| count)
            .sum();
        if chars > 0 {
            counts.push(chars as f64);
            first.get_or_insert(line);
        }
    }
    if counts.len() < MIN_MEASURED_LINES {
        return Vec::new();
    }
    counts.sort_by(f64::total_cmp);
    let median = counts[counts.len() / 2];
    let (too_long, too_short) =
        (median > MAX_CHARS_PER_LINE, median < MIN_CHARS_PER_LINE);
    if !too_long && !too_short {
        return Vec::new();
    }
    let line = first.unwrap();
    let mut diagnostic = Diagnostic::new(
        LINE_LENGTH,
        Severity::Warning,
        format!(
            "justified body lines hold about {median:.0} characters; \
             {MIN_CHARS_PER_LINE}-{MAX_CHARS_PER_LINE} reads comfortably"
        ),
    );
    diagnostic.loc = diag::resolve_span(world, line.span);
    diagnostic.page = Some(line.page + 1);
    diagnostic.measured = Some(median);
    diagnostic.limit =
        Some(if too_long { MAX_CHARS_PER_LINE } else { MIN_CHARS_PER_LINE });
    diagnostic.unit = Some("chars");
    diagnostic.hints.push(if too_long {
        "use a larger font size, wider margins or two columns".into()
    } else {
        "use a smaller font size, narrower margins or fewer columns, or set the text \
         ragged (justify: false)"
            .into()
    });
    vec![diagnostic]
}
