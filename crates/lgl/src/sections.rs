//! How sections share a page: a section beside a taller one that leaves a
//! hole above the next section of its column (L098), and a footer line set
//! as the last block of the flow, floating above an empty band (L099).
//! Measured in batch 19 on 821 stored documents.

use std::collections::{BTreeMap, BTreeSet};

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const HOLE_BESIDE_SECTION: &str = "L098.hole-beside-section";
pub const FLOATING_FOOTER: &str = "L099.floating-footer";

/// A section's hole counts from this many body sizes and this share of its
/// row. Measured: the two holes reported (6.3 and 9.4 sizes) read as torn
/// layouts; a 4.2-size hole in an approved job posting did not.
const MIN_HOLE_EM: f64 = 5.0;
const MIN_HOLE_SHARE: f64 = 0.25;

/// A section has at least this many lines of text.
const MIN_SECTION_LINES: usize = 3;

/// The next section of the column starts within this many body sizes under
/// the row.
const MAX_NEXT_GAP_EM: f64 = 4.0;

/// A footer line floats when the empty band under it takes at least this
/// share of the page height. Measured: the three floating footers of
/// product sheets left 17% to 22%.
const MIN_FOOTER_EMPTY_SHARE: f64 = 0.15;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let body = crate::titles::body_size(geometry).unwrap_or(10.0);
    let mut out = hole_beside_section(world, geometry, body);
    out.extend(floating_footer(world, geometry, body));
    out
}

fn nonblank(item: &Item) -> bool {
    item.kind == ItemKind::Text && item.text.as_deref().is_some_and(|t| !t.trim().is_empty())
}

/// Whether the element or one of its ancestors has one of the names.
fn inside(geometry: &Geometry, mut elem: Option<usize>, names: &[&str]) -> bool {
    while let Some(index) = elem {
        if names.contains(&geometry.elements[index].name) {
            return true;
        }
        elem = geometry.elements[index].parent;
    }
    false
}

/// Sections side by side in a row of a grid (not a table): one ends far
/// above the taller one, while the next section of its own column waits under
/// the row (sections paired by rows instead of stacked by columns).
fn hole_beside_section(world: &LabWorld, geometry: &Geometry, body: f64) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let containers = &geometry.containers;
    // The items inside each cell, nested cells included.
    let mut owned: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        let mut cursor = item.container;
        while let Some(current) = cursor {
            if containers[current].kind == "cell" {
                owned.entry(current).or_default().push(index);
            }
            cursor = containers[current].parent;
        }
    }
    let in_table = |cell: usize| inside(geometry, items[owned[&cell][0]].elem, &["table"]);
    let mut rows: BTreeMap<(usize, Option<usize>, i64), Vec<usize>> = BTreeMap::new();
    for &cell in owned.keys() {
        if in_table(cell) {
            continue;
        }
        let container = &containers[cell];
        rows.entry((container.page, container.parent, container.bbox.y0.round() as i64))
            .or_default()
            .push(cell);
    }
    let ink_end = |cell: usize| {
        owned[&cell]
            .iter()
            .map(|&k| &items[k])
            .filter(|i| matches!(i.kind, ItemKind::Text | ItemKind::Shape | ItemKind::Image))
            .map(|i| i.bbox.y1)
            .fold(f64::NEG_INFINITY, f64::max)
    };
    let mut out = Vec::new();
    for ((page, _, _), cells) in &rows {
        if cells.len() < 2 {
            continue;
        }
        let full = cells.iter().map(|&c| ink_end(c)).fold(f64::NEG_INFINITY, f64::max);
        for &cell in cells {
            let b = containers[cell].bbox;
            let end = ink_end(cell);
            if !end.is_finite() {
                continue;
            }
            let texts: Vec<&Item> =
                owned[&cell].iter().map(|&k| &items[k]).filter(|i| nonblank(i)).collect();
            let lines: BTreeSet<i64> =
                texts.iter().map(|t| t.bbox.y1.round() as i64).collect();
            let hole = full - end;
            if lines.len() < MIN_SECTION_LINES
                || hole < MIN_HOLE_EM * body
                || hole < MIN_HOLE_SHARE * (full - b.y0)
            {
                continue;
            }
            // A panel (a filled or framed cell, a ticket stub) is meant to
            // fill its row.
            let panel = containers[cell].visible
                || items.iter().any(|i| {
                    i.kind == ItemKind::Shape
                        && i.page == *page
                        && i.bbox.y0 <= b.y0 + 1.0
                        && i.bbox.x0 <= b.x0 + 1.0
                        && i.bbox.x1 >= b.x1 - 1.0
                        && i.bbox.height() >= 0.9 * b.height()
                        && i.bbox.width() >= 0.9 * b.width()
                });
            if panel {
                continue;
            }
            let next = owned.keys().any(|&other| {
                let n = containers[other].bbox;
                other != cell
                    && containers[other].page == *page
                    && !in_table(other)
                    && n.y0 >= b.y1 - 0.5
                    && n.y0 <= b.y1 + MAX_NEXT_GAP_EM * body
                    && (n.width() - b.width()).abs() <= 0.1 * n.width().max(b.width())
                    && n.x1.min(b.x1) - n.x0.max(b.x0) >= 0.9 * b.width()
            });
            if !next {
                continue;
            }
            let first = texts[0];
            let shown: String =
                first.text.as_deref().unwrap_or_default().trim().chars().take(30).collect();
            let mut diagnostic = Diagnostic::new(
                HOLE_BESIDE_SECTION,
                Severity::Warning,
                format!(
                    "the section starting \"{shown}\" on page {} ends {}pt above the taller \
                     section beside it, so the next section of its column waits under the \
                     row and a hole opens",
                    page + 1,
                    round1(hole)
                ),
            );
            diagnostic.loc =
                diag::resolve_span(world, geometry.span_for(first.span, first.elem));
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = Some(BBox { x0: b.x0, y0: end, x1: b.x1, y1: full }.rounded());
            diagnostic.measured = Some(round1(hole / body));
            diagnostic.limit = Some(MIN_HOLE_EM);
            diagnostic.unit = Some("body sizes");
            diagnostic.hints.push(
                "stack each column on its own instead of pairing sections by rows: \
                 grid(columns: 2, stack(spacing: 1.2em, first, third), stack(spacing: \
                 1.2em, second, fourth)), so a section follows the one above it; keep \
                 full-width content below the grid"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}

/// A footer line (company, address, codes under a rule) set as the last
/// block of the flow on a page without a footer, with an empty band under it.
fn floating_footer(world: &LabWorld, geometry: &Geometry, body: f64) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let kind = |item: &Item| item.region.map(|r| geometry.regions[r].kind);
    let mut out = Vec::new();
    for (index, page) in geometry.pages.iter().enumerate() {
        let Some(region) =
            geometry.regions.iter().find(|r| r.page == index && r.kind == "page-body")
        else {
            continue;
        };
        let r = region.bbox;
        if items.iter().any(|i| i.page == index && nonblank(i) && kind(i) == Some("footer")) {
            continue;
        }
        let content: Vec<&Item> = items
            .iter()
            .filter(|i| {
                i.page == index
                    && kind(i) == Some("page-body")
                    && match i.kind {
                        ItemKind::Text => nonblank(i),
                        // Page backgrounds and frames are not content.
                        ItemKind::Shape => i.bbox.height() <= 0.5 * r.height(),
                        ItemKind::Image => true,
                        ItemKind::Link => false,
                    }
            })
            .collect();
        let last = content.iter().map(|i| i.bbox.y1).fold(f64::NEG_INFINITY, f64::max);
        if !last.is_finite() {
            continue;
        }
        let empty = r.y1 - last;
        if empty < MIN_FOOTER_EMPTY_SHARE * page.height {
            continue;
        }
        let texts: Vec<&&Item> = content.iter().filter(|i| i.kind == ItemKind::Text).collect();
        let tail: Vec<&&&Item> =
            texts.iter().filter(|t| t.bbox.y1 > last - 2.6 * body).collect();
        if tail.is_empty() || tail.iter().any(|t| t.font_size.unwrap_or(0.0) >= body - 0.05) {
            continue;
        }
        let top = tail.iter().map(|t| t.bbox.y0).fold(f64::INFINITY, f64::min);
        let Some(rule) = content.iter().find(|s| {
            s.kind == ItemKind::Shape
                && s.bbox.height() < 1.5
                && s.bbox.width() >= 0.8 * r.width()
                && top - 2.5 * body <= s.bbox.y0
                && s.bbox.y0 <= top
                && !inside(geometry, s.elem, &["table", "grid", "figure"])
        }) else {
            continue;
        };
        if !texts.iter().any(|t| t.bbox.y1 < rule.bbox.y0 - 1.0) {
            continue;
        }
        let first = tail[0];
        let shown: String =
            first.text.as_deref().unwrap_or_default().trim().chars().take(30).collect();
        let mut diagnostic = Diagnostic::new(
            FLOATING_FOOTER,
            Severity::Warning,
            format!(
                "the footer line \"{shown}\" on page {} ends the content {}pt above the \
                 bottom margin, so it floats in the page instead of closing it",
                index + 1,
                round1(empty)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, geometry.span_for(first.span, first.elem));
        diagnostic.page = Some(index + 1);
        diagnostic.bbox = Some(rule.bbox.union(&first.bbox).rounded());
        diagnostic.measured = Some(round1(empty / page.height));
        diagnostic.limit = Some(MIN_FOOTER_EMPTY_SHARE);
        diagnostic.unit = Some("page share");
        diagnostic.hints.push(
            "set footer lines (company, address, product codes) in the page footer, \
             page(footer: ..), so they sit at the bottom of the page; the content above \
             stays where it is"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}
