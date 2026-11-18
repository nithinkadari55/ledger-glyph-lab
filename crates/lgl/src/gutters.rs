//! Row lines broken at a column gutter: a table or grid that draws its row
//! lines as cell borders while its columns have a gutter, so every row line
//! stops at the gap and starts again after it. Measured in batch 19 on 821
//! stored documents: 30 documents (catalogue cards, menus, checklists,
//! release notes) printed broken row lines.

use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const BROKEN_ROW_LINE: &str = "L100.broken-row-line";

/// A gap between two pieces of one row line, in points: the column gutter.
const MIN_GAP_PT: f64 = 2.0;
const MAX_GAP_PT: f64 = 30.0;

/// A line piece is thinner than this and at least this long, in points.
const LINE_MAX_HEIGHT_PT: f64 = 1.5;
const LINE_MIN_WIDTH_PT: f64 = 15.0;

/// Rows of a table broken at the same gap before it counts.
const MIN_BROKEN_ROWS: usize = 2;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let thin = |i: &Item| i.kind == ItemKind::Shape && i.bbox.height() < LINE_MAX_HEIGHT_PT;
    // Line pieces drawn by a table or grid itself (its cell borders), by row.
    let mut rows: BTreeMap<(usize, usize, i64), Vec<&Item>> = BTreeMap::new();
    for item in items {
        if !thin(item) || item.bbox.width() < LINE_MIN_WIDTH_PT {
            continue;
        }
        let Some(elem) = item.elem else { continue };
        if !matches!(geometry.elements[elem].name, "table" | "grid") {
            continue;
        }
        rows.entry((item.page, elem, (item.bbox.y0 * 2.0).round() as i64))
            .or_default()
            .push(item);
    }
    // The side line of a box meets the end of a piece: a frame, not a row line.
    let side = |page: usize, x: f64, y: f64| {
        items.iter().any(|o| {
            o.kind == ItemKind::Shape
                && o.page == page
                && o.bbox.width() < 1.5
                && o.bbox.height() > 4.0
                && ((o.bbox.x0 + o.bbox.x1) / 2.0 - x).abs() < 1.5
                && o.bbox.y0 - 1.5 <= y
                && y <= o.bbox.y1 + 1.5
        })
    };
    let mut broken: BTreeMap<(usize, usize, i64, i64), Vec<f64>> = BTreeMap::new();
    for ((page, elem, _), mut pieces) in rows {
        pieces.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
        for pair in pieces.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let gap = b.bbox.x0 - a.bbox.x1;
            if !(MIN_GAP_PT..=MAX_GAP_PT).contains(&gap)
                || (a.bbox.height() - b.bbox.height()).abs() > 0.1
                || a.color != b.color
            {
                continue;
            }
            // Another line drawn across the gap closes it.
            let bridged = items.iter().any(|o| {
                !std::ptr::eq(o, a)
                    && !std::ptr::eq(o, b)
                    && thin(o)
                    && o.page == page
                    && o.bbox.x0 <= a.bbox.x1 + 0.5
                    && o.bbox.x1 >= b.bbox.x0 - 0.5
                    && o.bbox.y0 <= a.bbox.y1
                    && o.bbox.y1 >= a.bbox.y0
            });
            if bridged || side(page, a.bbox.x1, a.bbox.y0) || side(page, b.bbox.x0, b.bbox.y0)
            {
                continue;
            }
            broken
                .entry((page, elem, a.bbox.x1.round() as i64, b.bbox.x0.round() as i64))
                .or_default()
                .push(a.bbox.y0);
        }
    }
    // One report per table: its broken rows at its widest-reaching gap.
    let mut tables: BTreeMap<(usize, usize), (usize, f64)> = BTreeMap::new();
    for ((page, elem, x1, x0), ys) in &broken {
        if ys.len() < MIN_BROKEN_ROWS {
            continue;
        }
        let entry = tables.entry((*page, *elem)).or_insert((0, 0.0));
        if ys.len() > entry.0 {
            *entry = (ys.len(), (x0 - x1) as f64);
        }
    }
    let mut out = Vec::new();
    for ((page, elem), (count, gap)) in tables {
        let element = &geometry.elements[elem];
        let mut diagnostic = Diagnostic::new(
            BROKEN_ROW_LINE,
            Severity::Warning,
            format!(
                "the row lines of a {} on page {} stop at the {}pt gap between its columns \
                 on {count} rows, so each line is printed in pieces",
                element.name,
                page + 1,
                round1(gap)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, element.span);
        diagnostic.elem = Some(element.name.into());
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = element.boxes.iter().find(|b| b.page == page).map(|b| b.bbox.rounded());
        diagnostic.measured = Some(count as f64);
        diagnostic.limit = Some(MIN_BROKEN_ROWS as f64);
        diagnostic.unit = Some("rows");
        diagnostic.hints.push(
            "draw row lines that run through the gutter with table.hline() (or \
             grid.hline()), or drop column-gutter and space the columns with the cells' \
             inset; cell strokes stop at the gutter"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}
