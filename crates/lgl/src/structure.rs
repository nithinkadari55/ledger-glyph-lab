use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const FIGURE_DISTANCE: &str = "L016.figure-far-from-reference";
pub const NUMBER_ALIGNMENT: &str = "L017.misaligned-numbers";
pub const FOOTNOTE_PAGE: &str = "L028.footnote-on-other-page";
pub const NARROW_COLUMN: &str = "L029.narrow-column";
pub const MIXED_PRECISION: &str = "L041.mixed-precision";

/// A table column narrower than this, whose text wraps into many lines, is too
/// narrow to read. Convention.
const MIN_COLUMN_PT: f64 = 60.0;
const MAX_NARROW_LINES: usize = 4;

/// How many pages a figure may be placed after (or before) its first
/// reference. Convention: the same or the following page.
const MAX_FIGURE_DISTANCE: usize = 1;

/// Numeric cells needed in a column before its alignment is judged.
const MIN_NUMERIC_CELLS: usize = 3;

/// How far the right edges of numbers in one column may differ.
const MAX_EDGE_SPREAD_PT: f64 = 1.0;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = figure_distance(world, geometry);
    out.extend(number_alignment(world, geometry));
    out.extend(footnote_pages(world, geometry));
    out.extend(narrow_columns(world, geometry));
    out.extend(ragged_columns(world, geometry));
    out
}

pub const RAGGED_COLUMN: &str = "L048.ragged-column";

/// How far the left and the right edges of a text column's cells may both
/// wander before the column has no straight edge at all (centred text of
/// different lengths, or mixed alignments).
const MAX_RAGGED_SPREAD_PT: f64 = 6.0;

/// Text cells, below the header, needed before a column is judged.
const MIN_TEXT_CELLS: usize = 3;

/// A text cell and the gaps between its edges and its ink.
struct TextCell {
    top: f64,
    left: f64,
    right: f64,
    item: usize,
}

fn spread(values: impl Iterator<Item = f64> + Clone) -> f64 {
    values.clone().fold(f64::NEG_INFINITY, f64::max)
        - values.fold(f64::INFINITY, f64::min)
}

/// Table columns of text whose cells share neither a left nor a right edge.
fn ragged_columns(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut cells: BTreeMap<usize, (String, f64, f64, usize)> = BTreeMap::new();
    for (index, item) in geometry.items.iter().enumerate() {
        if item.kind != ItemKind::Text {
            continue;
        }
        let Some(cell) = enclosing_cell(geometry, item.container) else { continue };
        let entry = cells.entry(cell).or_insert((
            String::new(),
            f64::INFINITY,
            f64::NEG_INFINITY,
            index,
        ));
        entry.0.push_str(item.text.as_deref().unwrap_or_default());
        entry.1 = entry.1.min(item.bbox.x0);
        entry.2 = entry.2.max(item.bbox.x1);
    }

    // Left and right gaps between each text cell and its ink, per column.
    let mut columns: BTreeMap<(usize, i64, i64), Vec<TextCell>> = BTreeMap::new();
    for (cell, (text, left, right, item)) in cells {
        if text.trim().is_empty() || is_numeric(&text) {
            continue;
        }
        let container = &geometry.containers[cell];
        let key = (
            container.page,
            (container.bbox.x0 * 2.0).round() as i64,
            (container.bbox.x1 * 2.0).round() as i64,
        );
        columns.entry(key).or_default().push(TextCell {
            top: container.bbox.y0,
            left: left - container.bbox.x0,
            right: container.bbox.x1 - right,
            item,
        });
    }

    let mut out = Vec::new();
    for ((page, _, _), mut column) in columns {
        // The topmost cell is usually a header, which may be set apart.
        column.sort_by(|a, b| a.top.total_cmp(&b.top));
        let body = &column[1.min(column.len())..];
        if body.len() < MIN_TEXT_CELLS {
            continue;
        }
        let left = spread(body.iter().map(|c| c.left));
        let right = spread(body.iter().map(|c| c.right));
        if left < MAX_RAGGED_SPREAD_PT || right < MAX_RAGGED_SPREAD_PT {
            continue;
        }
        let first = &geometry.items[body[0].item];
        let mut diagnostic = Diagnostic::new(
            RAGGED_COLUMN,
            Severity::Warning,
            format!(
                "a table column of text on page {} has no straight edge: its cells start \
                 {}pt apart and end {}pt apart, as happens when text of different lengths \
                 is centred",
                page + 1,
                round1(left),
                round1(right)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, first.span);
        diagnostic.elem = Some("cell".into());
        diagnostic.page = Some(page + 1);
        diagnostic.measured = Some(round1(left.min(right)));
        diagnostic.limit = Some(MAX_RAGGED_SPREAD_PT);
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "align text columns left (and numeric columns right), e.g. table(align: \
             (left, left, right)); start numbered labels at a common left edge"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn first_page(geometry: &Geometry, element: usize) -> Option<usize> {
    geometry.elements[element].boxes.iter().map(|b| b.page).min()
}

fn figure_distance(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, figure) in geometry.elements.iter().enumerate() {
        if figure.name != "figure" {
            continue;
        }
        let Some(label) = &figure.label else { continue };
        let Some(figure_page) = first_page(geometry, index) else { continue };
        let Some(reference) = geometry
            .elements
            .iter()
            .enumerate()
            .find(|(_, e)| e.name == "ref" && e.target.as_ref() == Some(label))
            .map(|(i, _)| i)
        else {
            continue;
        };
        let Some(reference_page) = first_page(geometry, reference) else { continue };
        if figure_page.abs_diff(reference_page) <= MAX_FIGURE_DISTANCE {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            FIGURE_DISTANCE,
            Severity::Warning,
            format!(
                "figure <{label}> is on page {} but first referenced on page {}",
                figure_page + 1,
                reference_page + 1
            ),
        );
        diagnostic.loc = diag::resolve_span(world, figure.span);
        diagnostic.elem = Some("figure".into());
        diagnostic.label = Some(label.clone());
        diagnostic.page = Some(figure_page + 1);
        diagnostic.measured = Some(figure_page.abs_diff(reference_page) as f64);
        diagnostic.limit = Some(MAX_FIGURE_DISTANCE as f64);
        diagnostic.unit = Some("pages");
        diagnostic.hints.push(
            "move the figure next to the paragraph that first references it, make it \
             float (placement: auto), or reduce its size so it fits sooner"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

struct NumericCell {
    bbox: BBox,
    left: f64,
    right: f64,
    item: usize,
    cell: usize,
    decimals: Option<usize>,
    unit: String,
}

/// The unit or currency signs of a number, such as `%` or `£`. Numbers that
/// share one measure the same quantity, so they share a precision; a column
/// of bare numbers may list different quantities (a specification table).
fn unit(text: &str) -> String {
    text.chars().filter(|c| "%$€£¥".contains(*c)).collect()
}

/// Digits after the decimal point, or `None` when the text has several points
/// (a version or an address rather than a decimal number).
fn decimals(text: &str) -> Option<usize> {
    let mut parts = text.split('.');
    parts.next()?;
    let Some(fraction) = parts.next() else { return Some(0) };
    if parts.next().is_some() {
        return None;
    }
    Some(fraction.chars().take_while(|c| c.is_ascii_digit()).count())
}

fn is_numeric(text: &str) -> bool {
    let text = text.trim();
    text.chars().any(|c| c.is_ascii_digit())
        && text.chars().all(|c| {
            c.is_ascii_digit() || " .,%+-−()$€£¥'\u{a0}\u{202f}\u{2009}".contains(c)
        })
}

fn number_alignment(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut cells: BTreeMap<usize, (String, f64, f64, usize)> = BTreeMap::new();
    for (index, item) in geometry.items.iter().enumerate() {
        if item.kind != ItemKind::Text {
            continue;
        }
        let Some(cell) = enclosing_cell(geometry, item.container) else { continue };
        let entry = cells.entry(cell).or_insert((
            String::new(),
            f64::INFINITY,
            f64::NEG_INFINITY,
            index,
        ));
        entry.0.push_str(item.text.as_deref().unwrap_or_default());
        entry.1 = entry.1.min(item.bbox.x0);
        entry.2 = entry.2.max(item.bbox.x1);
    }

    let mut columns: BTreeMap<(usize, i64, i64), Vec<NumericCell>> = BTreeMap::new();
    for (cell, (text, left, right, item)) in cells {
        if !is_numeric(&text) {
            continue;
        }
        let container = &geometry.containers[cell];
        let key = (
            container.page,
            (container.bbox.x0 * 2.0).round() as i64,
            (container.bbox.x1 * 2.0).round() as i64,
        );
        columns.entry(key).or_default().push(NumericCell {
            bbox: container.bbox,
            left,
            right,
            item,
            cell,
            decimals: decimals(&text),
            unit: unit(&text),
        });
    }

    let mut out = Vec::new();
    for ((page, _, _), cells) in columns {
        if cells.len() < MIN_NUMERIC_CELLS {
            continue;
        }
        out.extend(mixed_precision(world, geometry, page, &cells));
        let spread = |f: fn(&NumericCell) -> f64| {
            let values = cells.iter().map(f);
            values.clone().fold(f64::NEG_INFINITY, f64::max)
                - values.fold(f64::INFINITY, f64::min)
        };
        let right_spread = spread(|c| c.right);
        if right_spread <= MAX_EDGE_SPREAD_PT {
            continue;
        }
        let left_aligned = spread(|c| c.left) <= MAX_EDGE_SPREAD_PT;
        let column = cells.iter().fold(cells[0].bbox, |acc, c| acc.union(&c.bbox));
        let first = &geometry.items[cells[0].item];
        let mut diagnostic = Diagnostic::new(
            NUMBER_ALIGNMENT,
            Severity::Warning,
            format!(
                "numbers in a table column on page {} are {}; their right edges differ \
                 by {}pt, so digits do not line up",
                page + 1,
                if left_aligned { "left-aligned" } else { "not aligned" },
                round1(right_spread)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, first.span);
        diagnostic.elem = Some("cell".into());
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = Some(column.rounded());
        diagnostic.measured = Some(round1(right_spread));
        diagnostic.limit = Some(MAX_EDGE_SPREAD_PT);
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "right-align numeric columns (e.g. table(align: (left, right, ...))) and use \
             the same number of decimal places"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn mixed_precision(
    world: &LabWorld,
    geometry: &Geometry,
    page: usize,
    cells: &[NumericCell],
) -> Option<Diagnostic> {
    if cells[0].unit.is_empty() || cells.iter().any(|c| c.unit != cells[0].unit) {
        return None;
    }
    let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
    for decimals in cells.iter().filter_map(|c| c.decimals) {
        *counts.entry(decimals).or_default() += 1;
    }
    if counts.len() < 2 {
        return None;
    }
    let (common, _) = counts.iter().max_by_key(|(_, n)| **n)?;
    let odd = cells.iter().find(|c| c.decimals != Some(*common))?;
    let column = cells.iter().fold(cells[0].bbox, |acc, c| acc.union(&c.bbox));
    let item = &geometry.items[odd.item];
    let places: Vec<String> = counts.keys().map(|d| d.to_string()).collect();
    let mut diagnostic = Diagnostic::new(
        MIXED_PRECISION,
        Severity::Warning,
        format!(
            "numbers in a table column on page {} use {} decimal places, so their \
             decimal points do not line up",
            page + 1,
            places.join(" and ")
        ),
    );
    diagnostic.loc = diag::resolve_span(world, item.span)
        .or_else(|| diag::resolve_span(world, geometry.containers[odd.cell].span));
    diagnostic.elem = Some("cell".into());
    diagnostic.page = Some(page + 1);
    diagnostic.bbox = Some(column.rounded());
    diagnostic
        .hints
        .push(format!("write every number in the column with {common} decimal place(s)"));
    Some(diagnostic)
}

/// The nearest enclosing table or grid cell.
pub fn enclosing_cell(geometry: &Geometry, mut cursor: Option<usize>) -> Option<usize> {
    while let Some(index) = cursor {
        if geometry.containers[index].kind == "cell" {
            return Some(index);
        }
        cursor = geometry.containers[index].parent;
    }
    None
}

fn footnote_pages(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let pages_of = |name: &str| -> Vec<(usize, Option<usize>)> {
        geometry
            .elements
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name == name && !has_ancestor(geometry, e.parent, "entry"))
            .map(|(i, _)| (i, first_page(geometry, i)))
            .collect()
    };
    let markers = pages_of("footnote");
    let entries = pages_of("entry");
    if markers.len() != entries.len() {
        return Vec::new();
    }
    markers
        .iter()
        .zip(&entries)
        .filter_map(|(&(marker, marker_page), &(_, entry_page))| {
            let (marker_page, entry_page) = (marker_page?, entry_page?);
            (marker_page != entry_page).then_some((marker, marker_page, entry_page))
        })
        .map(|(marker, marker_page, entry_page)| {
            let mut out = Diagnostic::new(
                FOOTNOTE_PAGE,
                Severity::Warning,
                format!(
                    "footnote marker is on page {} but its text is on page {}",
                    marker_page + 1,
                    entry_page + 1
                ),
            );
            out.loc = diag::resolve_span(world, geometry.elements[marker].span);
            out.elem = Some("footnote".into());
            out.page = Some(marker_page + 1);
            out.hints.push(
                "shorten the footnote text, or move the footnote earlier in the paragraph"
                    .into(),
            );
            out
        })
        .collect()
}

fn has_ancestor(geometry: &Geometry, mut cursor: Option<usize>, name: &str) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].name == name {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

fn narrow_columns(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut lines_per_cell: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, line) in geometry.lines.iter().enumerate() {
        if is_wrapped(line)
            && let Some(cell) = enclosing_cell(geometry, line.container)
        {
            lines_per_cell.entry(cell).or_default().push(index);
        }
    }

    let mut worst: BTreeMap<(usize, i64, i64), (usize, usize)> = BTreeMap::new();
    for (cell, lines) in lines_per_cell {
        let container = &geometry.containers[cell];
        if container.bbox.width() >= MIN_COLUMN_PT || lines.len() < MAX_NARROW_LINES {
            continue;
        }
        let key = (
            container.page,
            (container.bbox.x0 * 2.0).round() as i64,
            (container.bbox.x1 * 2.0).round() as i64,
        );
        let entry = worst.entry(key).or_insert((cell, lines[0]));
        if lines.len() > lines_per_cell_len(geometry, entry.0) {
            *entry = (cell, lines[0]);
        }
    }

    worst
        .into_values()
        .map(|(cell, line)| {
            let container = &geometry.containers[cell];
            let count = lines_per_cell_len(geometry, cell);
            let mut out = Diagnostic::new(
                NARROW_COLUMN,
                Severity::Warning,
                format!(
                    "a table column on page {} is only {}pt wide, so its text wraps into \
                     {count} lines",
                    container.page + 1,
                    round1(container.bbox.width())
                ),
            );
            out.loc = diag::resolve_span(world, geometry.lines[line].span);
            out.elem = Some("cell".into());
            out.page = Some(container.page + 1);
            out.bbox = Some(container.bbox.rounded());
            out.measured = Some(round1(container.bbox.width()));
            out.limit = Some(MIN_COLUMN_PT);
            out.unit = Some("pt");
            out.hints.push(
                "give the column more room (auto or a larger fr share), or shorten its \
                 content"
                    .into(),
            );
            out
        })
        .collect()
}

fn lines_per_cell_len(geometry: &Geometry, cell: usize) -> usize {
    geometry
        .lines
        .iter()
        .filter(|line| {
            is_wrapped(line) && enclosing_cell(geometry, line.container) == Some(cell)
        })
        .count()
}

/// Whether a line was filled by wrapping text, as opposed to a short line such
/// as one of several stacked labels.
fn is_wrapped(line: &crate::geom::Line) -> bool {
    line.natural >= 0.6 * line.available
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use typst::syntax::Span;

    use super::*;
    use crate::geom::{Element, PageBox};
    use crate::world::WorldOptions;

    fn element(name: &'static str, page: usize) -> Element {
        let bbox = BBox { x0: 0.0, y0: 0.0, x1: 10.0, y1: 10.0 };
        Element {
            name,
            label: None,
            target: None,
            parent: None,
            boxes: vec![PageBox { page, bbox }],
            span: Span::detached(),
        }
    }

    fn world() -> LabWorld {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/ok-basic.typ");
        LabWorld::new(&path, &WorldOptions::default()).unwrap()
    }

    #[test]
    fn footnote_entry_on_later_page_is_reported() {
        let geometry = Geometry {
            elements: vec![element("footnote", 0), element("entry", 1)],
            ..Geometry::default()
        };
        let out = footnote_pages(&world(), &geometry);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].rule, FOOTNOTE_PAGE);
        assert_eq!(out[0].page, Some(1));
    }

    #[test]
    fn footnote_entry_on_same_page_is_fine() {
        let geometry = Geometry {
            elements: vec![element("footnote", 2), element("entry", 2)],
            ..Geometry::default()
        };
        assert!(footnote_pages(&world(), &geometry).is_empty());
    }
}
