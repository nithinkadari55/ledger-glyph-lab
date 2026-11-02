use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, Line, round1};
use crate::structure::enclosing_cell;
use crate::typography::paragraph_of;
use crate::world::LabWorld;

pub const LEADING: &str = "L033.leading-ratio";
pub const TOP_GAP: &str = "L034.top-gap";
pub const ROW_BASELINES: &str = "L035.row-baselines";
pub const EQUATION_NUMBER: &str = "L036.equation-number-gap";
pub const WRAPPED_CODE: &str = "L037.wrapped-code";

/// Comfortable line pitch divided by font size for running text. Convention.
const MIN_LEADING: f64 = 1.1;
const MAX_LEADING: f64 = 1.8;

/// How much deeper than usual the first content of a page may start.
const MAX_EXTRA_TOP_PT: f64 = 12.0;

/// How far the baselines of single-line cells in one row may differ.
const MAX_ROW_SPREAD_PT: f64 = 0.5;

/// The smallest gap between an equation and its number, relative to the
/// number's font size. Convention.
const MIN_NUMBER_GAP_EM: f64 = 0.5;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = leading(world, geometry);
    out.extend(top_gaps(geometry));
    out.extend(row_baselines(world, geometry));
    out.extend(equation_numbers(world, geometry));
    out.extend(wrapped_code(world, geometry));
    out.extend(cramped_lines(world, geometry));
    out
}

/// The text items whose middle lies within a line's box.
fn items_on<'a>(
    geometry: &'a Geometry,
    line: &'a Line,
) -> impl Iterator<Item = &'a Item> {
    geometry.items.iter().filter(move |item| {
        let middle = (item.bbox.y0 + item.bbox.y1) / 2.0;
        item.page == line.page
            && item.kind == ItemKind::Text
            && line.bbox.y0 <= middle
            && middle <= line.bbox.y1
            && item.bbox.x0 < line.bbox.x1
            && line.bbox.x0 < item.bbox.x1
    })
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied()
}

fn leading(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut ratios = Vec::new();
    let mut first = None;
    let lines = &geometry.lines;
    for pair in lines.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let par = paragraph_of(geometry, a.elem);
        if par.is_none()
            || par != paragraph_of(geometry, b.elem)
            || a.page != b.page
            || (a.bbox.x0 - b.bbox.x0).abs() > 0.5
            || !geometry.is_body(a.region)
        {
            continue;
        }
        let Some(size) =
            median(items_on(geometry, a).filter_map(|i| i.font_size).collect())
        else {
            continue;
        };
        if size > 0.0 {
            ratios.push((b.bbox.y0 - a.bbox.y0) / size);
            first.get_or_insert(a);
        }
    }
    if ratios.len() < 5 {
        return Vec::new();
    }
    let Some(ratio) = median(ratios) else { return Vec::new() };
    if (MIN_LEADING..=MAX_LEADING).contains(&ratio) {
        return Vec::new();
    }
    let line = first.unwrap();
    let tight = ratio < MIN_LEADING;
    let mut out = Diagnostic::new(
        LEADING,
        Severity::Warning,
        format!(
            "running text lines are {:.2} times the font size apart, which is too {}; \
             {MIN_LEADING}-{MAX_LEADING} reads comfortably",
            ratio,
            if tight { "tight" } else { "loose" }
        ),
    );
    out.loc = diag::resolve_span(world, line.span);
    out.page = Some(line.page + 1);
    out.measured = Some((ratio * 100.0).round() / 100.0);
    out.limit = Some(if tight { MIN_LEADING } else { MAX_LEADING });
    out.unit = Some("ratio");
    out.hints.push(if tight {
        "increase par(leading: ..)".into()
    } else {
        "decrease par(leading: ..)".into()
    });
    vec![out]
}

fn top_gaps(geometry: &Geometry) -> Vec<Diagnostic> {
    let mut offsets = Vec::new();
    for (index, region) in geometry.regions.iter().enumerate() {
        if region.kind != "page-body" {
            continue;
        }
        let top = geometry
            .items
            .iter()
            .filter(|item| item.region == Some(index) && item.kind != ItemKind::Link)
            .filter_map(Item::visible)
            .map(|bbox| bbox.y0)
            .min_by(f64::total_cmp);
        if let Some(top) = top {
            offsets.push((region.page, top - region.bbox.y0, region.bbox));
        }
    }
    if offsets.len() < 3 {
        return Vec::new();
    }
    let Some(usual) = median(offsets.iter().map(|o| o.1).collect()) else {
        return Vec::new();
    };
    offsets
        .into_iter()
        .filter(|&(page, offset, _)| page > 0 && offset > usual + MAX_EXTRA_TOP_PT)
        .map(|(page, offset, body)| {
            let mut out = Diagnostic::new(
                TOP_GAP,
                Severity::Warning,
                format!(
                    "content on page {} starts {}pt below the top of the text area, \
                     {}pt deeper than on other pages",
                    page + 1,
                    round1(offset),
                    round1(offset - usual)
                ),
            );
            out.page = Some(page + 1);
            out.bbox = Some(BBox { y1: body.y0 + offset, ..body }.rounded());
            out.measured = Some(round1(offset));
            out.limit = Some(round1(usual + MAX_EXTRA_TOP_PT));
            out.unit = Some("pt");
            out.hints
                .push("remove manual vertical space (v) at the start of the page".into());
            out
        })
        .collect()
}

fn row_baselines(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut lines_per_cell: BTreeMap<usize, Vec<&Line>> = BTreeMap::new();
    for line in &geometry.lines {
        if let Some(cell) = enclosing_cell(geometry, line.container) {
            lines_per_cell.entry(cell).or_default().push(line);
        }
    }

    let mut rows: BTreeMap<(usize, i64), Vec<(f64, &Line)>> = BTreeMap::new();
    for (cell, lines) in &lines_per_cell {
        let [line] = lines.as_slice() else { continue };
        let container = &geometry.containers[*cell];
        let key = (container.page, (container.bbox.y0 * 2.0).round() as i64);
        rows.entry(key)
            .or_default()
            .push((line.bbox.y0 + line.baseline, line));
    }

    let mut out = Vec::new();
    for ((page, _), cells) in rows {
        if cells.len() < 2 {
            continue;
        }
        let low = cells.iter().map(|c| c.0).fold(f64::INFINITY, f64::min);
        let high = cells.iter().map(|c| c.0).fold(f64::NEG_INFINITY, f64::max);
        if high - low <= MAX_ROW_SPREAD_PT {
            continue;
        }
        let lowest = cells.iter().max_by(|a, b| a.0.total_cmp(&b.0)).unwrap().1;
        let mut diagnostic = Diagnostic::new(
            ROW_BASELINES,
            Severity::Warning,
            format!(
                "the baselines of a table row on page {} differ by {}pt",
                page + 1,
                round1(high - low)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, lowest.span);
        diagnostic.elem = Some("cell".into());
        diagnostic.page = Some(page + 1);
        diagnostic.measured = Some(round1(high - low));
        diagnostic.limit = Some(MAX_ROW_SPREAD_PT);
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "use the same text size and vertical alignment for all cells of a row".into(),
        );
        out.push(diagnostic);
    }
    out
}

fn is_number(text: &str) -> bool {
    let text = text.trim();
    text.len() >= 3
        && text.starts_with('(')
        && text.ends_with(')')
        && text[1..text.len() - 1]
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
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

fn equation_numbers(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, equation) in geometry.elements.iter().enumerate() {
        if equation.name != "equation" {
            continue;
        }
        let items: Vec<&Item> = geometry
            .items
            .iter()
            .filter(|i| {
                i.kind == ItemKind::Text && descends_from(geometry, i.elem, index)
            })
            .collect();
        let Some(number) = items
            .iter()
            .filter(|i| i.text.as_deref().is_some_and(is_number))
            .max_by(|a, b| a.bbox.x1.total_cmp(&b.bbox.x1))
        else {
            continue;
        };
        let body_right = items
            .iter()
            .filter(|i| !std::ptr::eq(**i, *number))
            .filter(|i| i.page == number.page)
            .filter(|i| i.bbox.y0 < number.bbox.y1 && number.bbox.y0 < i.bbox.y1)
            .map(|i| i.bbox.x1)
            .fold(f64::NEG_INFINITY, f64::max);
        if !body_right.is_finite() {
            continue;
        }
        let gap = number.bbox.x0 - body_right;
        let limit = MIN_NUMBER_GAP_EM * number.font_size.unwrap_or(10.0);
        if gap >= limit {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            EQUATION_NUMBER,
            if gap < 0.0 { Severity::Error } else { Severity::Warning },
            format!(
                "equation number {} on page {} is only {}pt from the equation",
                number.text.as_deref().unwrap_or_default().trim(),
                number.page + 1,
                round1(gap)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, equation.span);
        diagnostic.elem = Some("equation".into());
        diagnostic.label = equation.label.clone();
        diagnostic.page = Some(number.page + 1);
        diagnostic.measured = Some(round1(gap));
        diagnostic.limit = Some(round1(limit));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "split the equation over several lines (with \\ and & alignment) or shorten it"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn wrapped_code(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut per_line: BTreeMap<usize, usize> = BTreeMap::new();
    for line in &geometry.lines {
        let mut cursor = line.elem;
        while let Some(index) = cursor {
            if geometry.elements[index].name == "line" {
                *per_line.entry(index).or_default() += 1;
                break;
            }
            cursor = geometry.elements[index].parent;
        }
    }

    let mut per_block: BTreeMap<Option<usize>, (usize, usize)> = BTreeMap::new();
    for (&code_line, &count) in &per_line {
        if count > 1 {
            let entry = per_block
                .entry(geometry.elements[code_line].parent)
                .or_insert((0, code_line));
            entry.0 += 1;
        }
    }

    per_block
        .into_values()
        .map(|(count, first)| {
            let element = &geometry.elements[first];
            let page = element.boxes.first().map(|b| b.page + 1);
            let mut out = Diagnostic::new(
                WRAPPED_CODE,
                Severity::Warning,
                format!(
                    "{count} line(s) of a code block wrap{}",
                    page.map(|p| format!(" on page {p}")).unwrap_or_default()
                ),
            );
            out.loc = diag::resolve_span(world, element.span);
            out.elem = Some("raw".into());
            out.page = page;
            out.hints.push(
                "shorten the code lines, use a smaller code font size, or break long \
                 statements"
                    .into(),
            );
            out
        })
        .collect()
}

pub const CRAMPED: &str = "L038.cramped-lines";

/// The smallest clear space between the ink of a line and the ink of the line
/// below it, relative to the larger of their text sizes. Below this, lines
/// look like they touch.
const MIN_CLEARANCE_EM: f64 = 0.1;

/// Consecutive lines of text whose ink nearly touches: too little leading, or
/// too little space between a heading or title and the text around it.
pub fn cramped_lines(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut by_page: BTreeMap<usize, Vec<(&Line, BBox, f64)>> = BTreeMap::new();
    for line in &geometry.lines {
        let ink = items_on(geometry, line)
            .filter_map(|item| Some((item.visible()?, item.font_size?)))
            .fold(None, |acc: Option<(BBox, f64)>, (bbox, size)| match acc {
                Some((a, s)) => Some((a.union(&bbox), s.max(size))),
                None => Some((bbox, size)),
            });
        if let Some((bbox, size)) = ink {
            by_page.entry(line.page).or_default().push((line, bbox, size));
        }
    }

    let mut out = Vec::new();
    for (page, mut lines) in by_page {
        lines.sort_by(|a, b| a.0.bbox.y0.total_cmp(&b.0.bbox.y0));
        for (i, (upper, upper_ink, upper_size)) in lines.iter().enumerate() {
            // The nearest line below that shares horizontal space.
            let Some((lower, lower_ink, lower_size)) = lines[i + 1..].iter().find(|(l, ..)| {
                l.bbox.y0 > upper.bbox.y0 + 0.5
                    && l.bbox.x0 < upper.bbox.x1
                    && upper.bbox.x0 < l.bbox.x1
            }) else {
                continue;
            };
            let size = upper_size.max(*lower_size);
            let clearance = lower_ink.y0 - upper_ink.y1;
            if clearance >= MIN_CLEARANCE_EM * size {
                continue;
            }
            let mut diagnostic = Diagnostic::new(
                CRAMPED,
                Severity::Error,
                format!(
                    "a line on page {} nearly touches the line above it: only {}pt of clear \
                     space for {}pt text",
                    page + 1,
                    round1(clearance),
                    round1(size)
                ),
            );
            diagnostic.loc = diag::resolve_span(world, lower.span);
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = Some(upper_ink.union(lower_ink).rounded());
            diagnostic.measured = Some(round1(clearance));
            diagnostic.limit = Some(round1(MIN_CLEARANCE_EM * size));
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "give large text more leading (set par(leading: ..) in the large text's \
                 own size, e.g. inside text(size: ..)[..]) or more space between the \
                 blocks"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}
