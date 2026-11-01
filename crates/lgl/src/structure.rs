use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const FIGURE_DISTANCE: &str = "L016.figure-far-from-reference";
pub const NUMBER_ALIGNMENT: &str = "L017.misaligned-numbers";
pub const FOOTNOTE_PAGE: &str = "L028.footnote-on-other-page";
pub const NARROW_COLUMN: &str = "L029.narrow-column";

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
        });
    }

    let mut out = Vec::new();
    for ((page, _, _), cells) in columns {
        if cells.len() < MIN_NUMERIC_CELLS {
            continue;
        }
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

/// The nearest enclosing table or grid cell.
fn enclosing_cell(geometry: &Geometry, mut cursor: Option<usize>) -> Option<usize> {
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
        if let Some(cell) = enclosing_cell(geometry, line.container) {
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
        .filter(|line| enclosing_cell(geometry, line.container) == Some(cell))
        .count()
}
