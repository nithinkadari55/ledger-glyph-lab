//! A gap that tears a column, a box of set size or a whole page apart.
//! Ported unchanged from batch 9 (L093), where it was measured on 394
//! generated documents and 275 first drafts.

use std::collections::BTreeSet;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const COLUMN_HOLE: &str = "L093.hole-in-column";

/// A hole is at least this many times the next largest gap of its column,
/// and this many sizes of the largest text beside it (large text stands in
/// large space). Measured on 394 generated documents and 275 first drafts:
/// the holes of a poster are 3.4 and 4.9 times the next gap and 6 and 12
/// sizes; the largest gap of an even column is up to 2.8 times the next, and
/// gaps that stand out as much elsewhere are at most 2.3 sizes.
const HOLE_RATIO: f64 = 3.0;
const MIN_HOLE_EM: f64 = 4.0;

/// Rows of content a column needs before one of its gaps can stand out.
const MIN_HOLE_ROWS: usize = 4;

/// A box covering this share of the text area in both directions is the
/// page itself, where space may be spread between sections.
const PAGE_SHARE: f64 = 0.8;

/// On a whole page, a hole counts when the content after it covers at least
/// this share of the text area: less is a footer or a signature row pinned
/// to the bottom. Measured on 420 generated documents: signature rows and
/// footers cover up to 0.10; the content under the holes of three wedding
/// covers and under a figure floated to the bottom covers 0.23 to 0.54.
const PAGE_BELOW_SHARE: f64 = 0.15;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let Some(body) = crate::titles::body_size(geometry) else { return Vec::new() };
    column_holes(world, geometry, body)
}

fn grown(b: &BBox, by: f64) -> BBox {
    BBox {
        x0: b.x0 - by,
        y0: b.y0 - by,
        x1: b.x1 + by,
        y1: b.y1 + by,
    }
}

fn holds(b: &BBox, (x, y): (f64, f64)) -> bool {
    b.x0 <= x && x <= b.x1 && b.y0 <= y && y <= b.y1
}

fn excerpt(item: &Item) -> Option<String> {
    let text = item.text.as_deref()?.trim();
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(24).collect();
    Some(if chars.next().is_some() { format!("{head}…") } else { head })
}

/// Inside a column or a box of set size, one gap far larger than every other
/// gap, with more than one row of content after it: stretched space that
/// tears the content apart. A last row pinned to the bottom is a footer, and
/// a box as large as the page is the page.
///
/// The text area of a page is judged too, more strictly: space may be
/// spread between its sections, so only one gap that collects it all counts
/// (or two around a single row, such as an ornament), and only when a real
/// part of the page follows it.
fn column_holes(world: &LabWorld, geometry: &Geometry, body: f64) -> Vec<Diagnostic> {
    let mut holders: Vec<(usize, BBox, bool)> =
        geometry.columns.iter().map(|c| (c.page, c.bbox, false)).collect();
    holders.extend(
        geometry
            .containers
            .iter()
            .filter(|c| c.sized && matches!(c.kind, "cell" | "block" | "box"))
            .map(|c| (c.page, c.bbox, false)),
    );
    holders.extend(
        geometry
            .regions
            .iter()
            .filter(|r| r.kind == "page-body")
            .map(|r| (r.page, r.bbox, true)),
    );
    let mut seen = BTreeSet::new();
    let mut reported = BTreeSet::new();
    let mut out = Vec::new();
    for (page, holder, whole) in holders {
        let key = [holder.x0, holder.y0, holder.x1, holder.y1].map(|v| v.round() as i64);
        if !seen.insert((page, key, whole)) {
            continue;
        }
        let page_sized = geometry.regions.iter().any(|r| {
            r.kind == "page-body"
                && r.page == page
                && holder.height() >= PAGE_SHARE * r.bbox.height()
                && holder.width() >= PAGE_SHARE * r.bbox.width()
        });
        if page_sized && !whole {
            continue;
        }
        let area = grown(&holder, 0.5);
        let mut content: Vec<&Item> = geometry
            .items
            .iter()
            .filter(|i| {
                let b = &i.bbox;
                let shown = match i.kind {
                    ItemKind::Text => {
                        i.text.as_deref().is_some_and(|t| !t.trim().is_empty())
                    }
                    ItemKind::Shape => b.height() <= 0.9 * holder.height(),
                    ItemKind::Image => true,
                    ItemKind::Link => false,
                };
                shown
                    && i.page == page
                    && holds(&area, ((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0))
                    && b.height() <= holder.height() + 1.0
            })
            .collect();
        content.sort_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
        // Rows of content: where each starts and ends, its first item and its
        // largest text.
        let mut rows: Vec<(f64, f64, &Item, f64)> = Vec::new();
        for item in content {
            let size = item.font_size.unwrap_or_default();
            match rows.last_mut() {
                Some(row) if item.bbox.y0 <= row.1 + 0.5 => {
                    row.1 = row.1.max(item.bbox.y1);
                    row.3 = row.3.max(size);
                }
                _ => rows.push((item.bbox.y0, item.bbox.y1, item, size)),
            }
        }
        if rows.len() < MIN_HOLE_ROWS {
            continue;
        }
        let gaps: Vec<f64> = rows.windows(2).map(|w| w[1].0 - w[0].1).collect();
        let mut order: Vec<usize> = (0..gaps.len()).collect();
        order.sort_by(|a, b| gaps[*b].total_cmp(&gaps[*a]));
        // The largest gap, or on a page the two largest when they stand on
        // either side of one row and neither stands out alone.
        let pair = whole
            && order[0].abs_diff(order[1]) == 1
            && gaps[order[0]] < HOLE_RATIO * gaps[order[1]];
        let holes = &order[..if pair { 2 } else { 1 }];
        let next = order.get(holes.len()).map_or(0.0, |k| gaps[*k]);
        let (first, last) = holes
            .iter()
            .fold((usize::MAX, 0), |(low, high), &k| (low.min(k), high.max(k)));
        let least = |k: usize| {
            let size = body.max(rows[k].3).max(rows[k + 1].3);
            (HOLE_RATIO * next).max(MIN_HOLE_EM * size)
        };
        if last + 1 == gaps.len() || holes.iter().any(|&k| gaps[k] < least(k)) {
            continue;
        }
        let end = rows[last + 1..].iter().map(|row| row.1).fold(0.0, f64::max);
        if whole && end - rows[last + 1].0 < PAGE_BELOW_SHARE * holder.height() {
            continue;
        }
        let hole = gaps[first];
        let (top, bottom, after) = (rows[first].1, rows[last + 1].0, rows[last + 1].2);
        if !reported.insert((page, top.round() as i64, bottom.round() as i64)) {
            continue;
        }
        let place = match excerpt(after) {
            Some(text) => format!(" above \"{text}\""),
            None => String::new(),
        };
        let times = round1(gaps[holes[holes.len() - 1]] / next.max(0.1));
        let message = if !whole {
            format!(
                "a column or box on page {} has an empty gap of {}pt{place}, {times} times \
                 its next largest gap, so its content is torn apart",
                page + 1,
                round1(hole)
            )
        } else if pair {
            format!(
                "page {} has two empty gaps of {}pt and {}pt around a single line{place}, \
                 {times} times its next largest gap, so the page falls into two parts",
                page + 1,
                round1(gaps[first]),
                round1(gaps[last])
            )
        } else {
            format!(
                "page {} has an empty gap of {}pt{place}, {times} times its next largest \
                 gap, so the page falls into two parts",
                page + 1,
                round1(hole)
            )
        };
        let mut diagnostic = Diagnostic::new(COLUMN_HOLE, Severity::Warning, message);
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(after.span, after.elem));
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = Some(BBox { y0: top, y1: bottom, ..holder }.rounded());
        diagnostic.measured = Some(round1(hole));
        diagnostic.limit = Some(round1(least(first)));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(if whole {
            "share the spare space of the page between all its sections (a fixed gap \
             plus v(1fr) in every gap between sections) or let the content follow on; \
             one stretched space, also one on each side of an ornament, leaves a hole, \
             and so does a figure floated to the bottom (use placement: none)"
                .into()
        } else {
            "let the content follow on at its normal spacing: do not stretch space \
             (v(1fr)) inside a column or a box of set height; spread spare space only \
             between whole sections of the page, or move a section to another column"
                .into()
        });
        out.push(diagnostic);
    }
    out
}
