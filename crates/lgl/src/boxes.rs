use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::structure::enclosing_cell;
use crate::world::LabWorld;

pub const HOLLOW_BOX: &str = "L039.hollow-box";
pub const TOUCHING_CELLS: &str = "L040.touching-cells";

/// Label for boxes whose empty area is intended, such as space for a
/// signature or handwritten notes.
pub const ALLOW_EMPTY: &str = "allow-empty";

/// A framed box may be empty below or around its content by this much before
/// the empty area is a defect. Convention: one inch.
const MAX_EMPTY_PT: f64 = 72.0;

/// Content covering less than this share of a framed box's height leaves it
/// mostly hollow.
const MIN_FILLED_SHARE: f64 = 0.4;

/// A hollow box is an error when its empty area exceeds this share of the
/// page height, or when it has no content at all.
const ERROR_PAGE_SHARE: f64 = 0.25;

/// The thickest side stripe or rule that counts as decoration.
const MAX_STRIPE_PT: f64 = 6.0;

/// Text in neighbouring cells must be separated by at least this fraction of
/// the font size (about a word space).
const MIN_CELL_GAP_EM: f64 = 0.2;

/// Text runs overlapping by more than this fraction of the font size are
/// reported by L006 instead.
const OVERLAP_SLACK_EM: f64 = 0.25;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = hollow_boxes(world, geometry);
    out.extend(touching_cells(world, geometry));
    out.extend(cell_padding(world, geometry));
    out.extend(double_rules(world, geometry));
    out.extend(box_padding(world, geometry));
    out.extend(dangling_separators(world, geometry));
    out.extend(uneven_boxes(world, geometry));
    out
}

pub const DANGLING_SEPARATOR: &str = "L062.dangling-separator";

fn within_any(geometry: &Geometry, mut cursor: Option<usize>, names: &[&str]) -> bool {
    while let Some(index) = cursor {
        if names.contains(&geometry.elements[index].name) {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// Separator rules at the very top or bottom of a column, where they
/// separate nothing: the column break already does.
fn dangling_separators(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let visible = |i: &&crate::geom::Item| {
        matches!(i.kind, ItemKind::Text | ItemKind::Shape | ItemKind::Image)
            && i.visible().is_some()
    };
    let mut out = Vec::new();
    for rule in geometry.items.iter().filter(|i| horizontal_rule(i)) {
        let Some(column) = rule.column else { continue };
        if rule.bbox.width() < 60.0
            || within_any(
                geometry,
                rule.elem,
                &["table", "grid", "cell", "figure", "heading"],
            )
        {
            continue;
        }
        let r = &rule.bbox;
        let neighbours: Vec<&crate::geom::Item> = geometry
            .items
            .iter()
            .filter(visible)
            .filter(|i| {
                !std::ptr::eq(*i, rule)
                    && i.page == rule.page
                    && i.column == Some(column)
                    && i.bbox.x0 < r.x1
                    && r.x0 < i.bbox.x1
            })
            .collect();
        let above = neighbours.iter().any(|i| i.bbox.y1 <= r.y0 + 0.5);
        let below = neighbours.iter().any(|i| i.bbox.y0 >= r.y1 - 0.5);
        let side = match (above, below) {
            (false, true) => "top",
            (true, false) => "bottom",
            _ => continue,
        };
        let mut diagnostic = Diagnostic::new(
            DANGLING_SEPARATOR,
            Severity::Warning,
            format!(
                "a separator rule stands at the {side} of a column on page {} with \
                 nothing on its other side, so it separates nothing",
                rule.page + 1
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(rule.span, rule.elem));
        diagnostic.page = Some(rule.page + 1);
        diagnostic.bbox = Some(r.rounded());
        diagnostic.hints.push(
            "draw separators only between items (e.g. skip the rule before the first \
             article of a column), or let the article break to the next column without \
             its rule"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

pub const CELL_PADDING: &str = "L059.tight-cell-padding";
pub const DOUBLE_RULE: &str = "L060.double-rule";
pub const BOX_PADDING: &str = "L061.tight-box-padding";

/// Text in a table cell needs this much clear space to the row rules above
/// and below its ink; Typst's default inset leaves about 0.2em. Closer than
/// 0.08em is reported by L050 instead.
const MIN_CELL_CLEAR_EM: f64 = 0.15;

/// Two parallel rules of this distance with nothing between them read as a
/// doubled separator; closer pairs are a deliberate double line.
const DOUBLE_RULE_GAP_PT: std::ops::RangeInclusive<f64> = 3.0..=36.0;

/// Text in a filled box needs this much clear space to the top and bottom
/// edges of the fill.
const MIN_BOX_CLEAR_EM: f64 = 0.25;

/// A thin horizontal rule: a shape at most 1.5pt tall and 20pt wide.
fn horizontal_rule(item: &crate::geom::Item) -> bool {
    item.kind == ItemKind::Shape && item.bbox.height() < 1.5 && item.bbox.width() >= 20.0
}

fn nonblank_text(item: &crate::geom::Item) -> bool {
    item.kind == ItemKind::Text
        && item.font_size.is_some()
        && item.text.as_deref().is_some_and(|t| !t.trim().is_empty())
}

/// Tables whose rows are ruled so tightly that text nearly touches the rules.
fn cell_padding(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let rules: Vec<&crate::geom::Item> =
        geometry.items.iter().filter(|i| horizontal_rule(i)).collect();
    // The tightest text per table, keyed by the cell's parent container.
    let mut tables: BTreeMap<(usize, Option<usize>), (f64, f64, &crate::geom::Item)> =
        BTreeMap::new();
    for text in geometry.items.iter().filter(|i| nonblank_text(i)) {
        let Some(cell) = enclosing_cell(geometry, text.container) else { continue };
        let size = text.font_size.unwrap_or(10.0);
        let t = &text.bbox;
        for rule in rules.iter().filter(|r| r.page == text.page) {
            let r = &rule.bbox;
            let overlap = r.x1.min(t.x1) - r.x0.max(t.x0);
            if overlap < 0.2 * t.width() {
                continue;
            }
            for clear in [r.y0 - t.y1, t.y0 - r.y1] {
                if !(0.08 * size..MIN_CELL_CLEAR_EM * size).contains(&clear) {
                    continue;
                }
                let key = (text.page, geometry.containers[cell].parent);
                let entry = tables.entry(key).or_insert((clear, size, text));
                if clear / size < entry.0 / entry.1 {
                    *entry = (clear, size, text);
                }
            }
        }
    }
    tables
        .into_iter()
        .map(|((page, _), (clear, size, text))| {
            let mut diagnostic = Diagnostic::new(
                CELL_PADDING,
                Severity::Warning,
                format!(
                    "text in a table on page {} sits only {}pt from a row rule, too \
                     tight for {}pt text",
                    page + 1,
                    round1(clear),
                    round1(size)
                ),
            );
            diagnostic.loc =
                diag::resolve_span(world, geometry.span_for(text.span, text.elem));
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = text.visible().map(|b| b.rounded());
            diagnostic.measured = Some(round1(clear));
            diagnostic.limit = Some(round1(MIN_CELL_CLEAR_EM * size));
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "increase the table's vertical inset (e.g. inset: (x: 6pt, y: 5pt)) so \
                 text keeps clear of the row rules"
                    .into(),
            );
            diagnostic
        })
        .collect()
}

/// Two parallel horizontal rules of the same length with nothing between
/// them, such as a section's closing rule followed by the next section's
/// opening rule.
fn double_rules(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let rules: Vec<&crate::geom::Item> =
        geometry.items.iter().filter(|i| horizontal_rule(i)).collect();
    let mut out = Vec::new();
    for a in &rules {
        for b in &rules {
            let (ra, rb) = (&a.bbox, &b.bbox);
            if a.page != b.page || rb.y0 <= ra.y1 {
                continue;
            }
            let gap = rb.y0 - ra.y1;
            let overlap = ra.x1.min(rb.x1) - ra.x0.max(rb.x0);
            let (wa, wb) = (ra.width(), rb.width());
            if !DOUBLE_RULE_GAP_PT.contains(&gap)
                || overlap < 0.9 * wa.min(wb)
                || (wa - wb).abs() > 0.1 * wa.max(wb)
            {
                continue;
            }
            let band = BBox {
                x0: ra.x0.max(rb.x0),
                y0: ra.y1,
                x1: ra.x1.min(rb.x1),
                y1: rb.y0,
            };
            let empty = !geometry.items.iter().any(|i| {
                i.page == a.page
                    && !std::ptr::eq(i, *a)
                    && !std::ptr::eq(i, *b)
                    && i.kind != ItemKind::Link
                    && i.bbox.y0 < band.y1
                    && i.bbox.y1 > band.y0
                    && i.bbox.x0 < band.x1
                    && i.bbox.x1 > band.x0
            });
            if !empty {
                continue;
            }
            let mut diagnostic = Diagnostic::new(
                DOUBLE_RULE,
                Severity::Warning,
                format!(
                    "two rules on page {} run {}pt apart with nothing between them, so \
                     the separator is doubled",
                    a.page + 1,
                    round1(gap)
                ),
            );
            diagnostic.loc = diag::resolve_span(world, geometry.span_for(b.span, b.elem));
            diagnostic.page = Some(a.page + 1);
            diagnostic.bbox = Some(ra.union(rb).rounded());
            diagnostic.measured = Some(round1(gap));
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "keep one of the two rules: drop the closing rule of the previous \
                 section or the opening rule (or top border) of the next"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}

/// Filled boxes whose text nearly touches the top or bottom edge of the fill.
fn box_padding(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for container in &geometry.containers {
        if container.kind != "block" || !container.visible {
            continue;
        }
        let frame = &container.bbox;
        // Only a fill shows every edge; a border on one side does not.
        let filled = geometry.items.iter().any(|i| {
            i.page == container.page
                && i.kind == ItemKind::Shape
                && i.color.is_some()
                && is_frame(&i.bbox, frame)
        });
        if !filled {
            continue;
        }
        let inside: Vec<&crate::geom::Item> = geometry
            .items
            .iter()
            .filter(|i| {
                nonblank_text(i)
                    && i.page == container.page
                    && i.bbox.x0 >= frame.x0 - 1.0
                    && i.bbox.x1 <= frame.x1 + 1.0
                    && i.bbox.y0 >= frame.y0 - 1.0
                    && i.bbox.y1 <= frame.y1 + 1.0
            })
            .collect();
        let Some(largest) =
            inside.iter().filter_map(|i| i.font_size).max_by(f64::total_cmp)
        else {
            continue;
        };
        if frame.height() < 2.0 * largest {
            continue;
        }
        let top = inside.iter().min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
        let bottom = inside.iter().max_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1));
        let sides = [
            ("top", top, top.map(|t| t.bbox.y0 - frame.y0)),
            ("bottom", bottom, bottom.map(|t| frame.y1 - t.bbox.y1)),
        ];
        for (side, text, clear) in sides {
            let (Some(text), Some(clear)) = (text, clear) else { continue };
            let size = text.font_size.unwrap_or(10.0);
            if clear >= MIN_BOX_CLEAR_EM * size {
                continue;
            }
            let mut diagnostic = Diagnostic::new(
                BOX_PADDING,
                Severity::Warning,
                format!(
                    "{}pt text sits only {}pt from the {side} edge of its filled box on \
                     page {}",
                    round1(size),
                    round1(clear.max(0.0)),
                    container.page + 1
                ),
            );
            diagnostic.loc = diag::resolve_span(world, container.span);
            diagnostic.page = Some(container.page + 1);
            diagnostic.bbox = Some(frame.rounded());
            diagnostic.measured = Some(round1(clear));
            diagnostic.limit = Some(round1(MIN_BOX_CLEAR_EM * size));
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "increase the box's inset on that side, scaled to its largest text \
                 (e.g. inset: (top: 0.5em, ..))"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}

fn labelled(geometry: &Geometry, mut cursor: Option<usize>, label: &str) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].label.as_deref() == Some(label) {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// Whether `bbox` is the container's own fill or stroke.
fn is_frame(bbox: &BBox, frame: &BBox) -> bool {
    bbox.width() * bbox.height() >= 0.8 * frame.width() * frame.height()
}

/// Whether `bbox` is a decorative stripe or rule along a whole side of the
/// frame, such as the accent bar of a pull quote. It stretches with the box,
/// so it does not show how much content the box holds.
fn is_stripe(bbox: &BBox, frame: &BBox) -> bool {
    (bbox.width() <= MAX_STRIPE_PT && bbox.height() >= 0.9 * frame.height())
        || (bbox.height() <= MAX_STRIPE_PT && bbox.width() >= 0.9 * frame.width())
}

fn hollow_boxes(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut content: Vec<Option<BBox>> = vec![None; geometry.containers.len()];
    for item in &geometry.items {
        if item.kind == ItemKind::Link {
            continue;
        }
        let Some(visible) = item.visible() else { continue };
        let mut cursor = item.container;
        while let Some(index) = cursor {
            let container = &geometry.containers[index];
            if item.kind != ItemKind::Shape
                || !(is_frame(&visible, &container.bbox)
                    || is_stripe(&visible, &container.bbox))
            {
                content[index] = Some(match content[index] {
                    Some(existing) => existing.union(&visible),
                    None => visible,
                });
            }
            cursor = container.parent;
        }
    }

    // A box that holds nothing at all is a deliberate shape, unless it is the
    // continuation of a box whose content is on another page.
    let with_content: std::collections::HashSet<_> = geometry
        .containers
        .iter()
        .zip(&content)
        .filter(|(_, c)| c.is_some())
        .map(|(container, _)| container.span)
        .collect();

    let mut out = Vec::new();
    for (index, container) in geometry.containers.iter().enumerate() {
        if container.kind == "cell"
            || !container.visible
            || !geometry.is_body(container.region)
            || labelled(geometry, container.elem, ALLOW_EMPTY)
        {
            continue;
        }
        let height = container.bbox.height();
        let filled = content[index]
            .and_then(|c| c.intersect(&container.bbox))
            .map_or(0.0, |c| c.height());
        if content[index].is_none()
            && (container.span.is_detached() || !with_content.contains(&container.span))
        {
            continue;
        }
        let empty = height - filled;
        if empty < MAX_EMPTY_PT || filled >= MIN_FILLED_SHARE * height {
            continue;
        }
        let page_height = geometry.pages[container.page].height;
        let severity = if filled == 0.0 || empty >= ERROR_PAGE_SHARE * page_height {
            Severity::Error
        } else {
            Severity::Warning
        };
        let mut diagnostic = Diagnostic::new(
            HOLLOW_BOX,
            severity,
            if filled == 0.0 {
                format!(
                    "a framed box on page {} is {}pt tall and empty",
                    container.page + 1,
                    round1(height)
                )
            } else {
                format!(
                    "a framed box on page {} is {}pt tall but its content fills only {}pt",
                    container.page + 1,
                    round1(height),
                    round1(filled)
                )
            },
        );
        diagnostic.loc = diag::resolve_span(world, container.span);
        diagnostic.elem = container.elem.map(|e| geometry.elements[e].name.into());
        diagnostic.page = Some(container.page + 1);
        diagnostic.bbox = Some(container.bbox.rounded());
        diagnostic.measured = Some(round1(empty));
        diagnostic.limit = Some(MAX_EMPTY_PT);
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "remove the fixed height (height: 100%, 1fr or a large length) so the box \
             fits its content; to make boxes in a row equally tall, size the grid row \
             instead"
                .into(),
        );
        diagnostic.hints.push(format!(
            "if the empty area is intended, such as space for a signature, label the \
             box <{ALLOW_EMPTY}>"
        ));
        out.push(diagnostic);
    }
    out
}

struct CellText {
    item: usize,
    cell: usize,
    bbox: BBox,
    size: f64,
}

fn touching_cells(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut pages: BTreeMap<usize, Vec<CellText>> = BTreeMap::new();
    for (index, item) in geometry.items.iter().enumerate() {
        if item.kind != ItemKind::Text
            || item.text.as_deref().is_none_or(|t| t.trim().is_empty())
            || labelled(geometry, item.elem, crate::overlap::ALLOW_OVERLAP)
        {
            continue;
        }
        let (Some(size), Some(bbox)) = (item.font_size, item.visible()) else { continue };
        let Some(cell) = enclosing_cell(geometry, item.container) else { continue };
        pages.entry(item.page).or_default().push(CellText {
            item: index,
            cell,
            bbox,
            size,
        });
    }

    // A drop cap keeps its own gap to the text beside it.
    let drop_caps = crate::titles::drop_cap_cells(geometry);
    let mut seen = BTreeMap::new();
    for texts in pages.values() {
        for left in texts {
            for right in texts {
                if left.cell == right.cell
                    || drop_caps.contains(&left.cell)
                    || drop_caps.contains(&right.cell)
                {
                    continue;
                }
                let size = left.size.max(right.size);
                let gap = right.bbox.x0 - left.bbox.x1;
                if gap >= MIN_CELL_GAP_EM * size
                    || gap < -OVERLAP_SLACK_EM * size
                    || right.bbox.x1 <= left.bbox.x1
                {
                    continue;
                }
                let shared =
                    left.bbox.y1.min(right.bbox.y1) - left.bbox.y0.max(right.bbox.y0);
                if shared < 0.5 * left.bbox.height().min(right.bbox.height()) {
                    continue;
                }
                let key = (left.cell, right.cell);
                let entry = seen.entry(key).or_insert((left.item, right.item, gap, size));
                if gap < entry.2 {
                    *entry = (left.item, right.item, gap, size);
                }
            }
        }
    }

    seen.into_values()
        .map(|(left, right, gap, size)| {
            let a = &geometry.items[left];
            let b = &geometry.items[right];
            let mut out = Diagnostic::new(
                TOUCHING_CELLS,
                Severity::Error,
                format!(
                    "text in neighbouring cells on page {} touches: \"{}\" ends {}pt from \
                     \"{}\"",
                    a.page + 1,
                    excerpt(a.text.as_deref()),
                    round1(gap),
                    excerpt(b.text.as_deref())
                ),
            );
            out.loc = diag::resolve_span(world, geometry.span_for(b.span, b.elem));
            out.page = Some(a.page + 1);
            out.bbox = Some(a.bbox.union(&b.bbox).rounded());
            out.measured = Some(round1(gap));
            out.limit = Some(round1(MIN_CELL_GAP_EM * size));
            out.unit = Some("pt");
            out.hints.push(
                "add column-gutter (or inset) to the grid or table so its columns are \
                 visibly separated"
                    .into(),
            );
            out
        })
        .collect()
}

fn excerpt(text: Option<&str>) -> String {
    let text = text.unwrap_or_default().trim();
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(24).collect();
    if chars.next().is_some() { format!("{head}…") } else { head }
}

pub const UNEVEN_BOXES: &str = "L069.uneven-boxes";

/// Boxes side by side in a row read as one band only at equal height.
/// Measured on 115 generated documents: 23 of 24 neighbouring box pairs
/// differ by less than 3%.
const MIN_HEIGHT_RATIO: f64 = 0.9;

/// Boxes at most this far apart, with tops level, stand in one row.
const MAX_ROW_GUTTER_PT: f64 = 40.0;

/// Boxes lower than this are chips and labels, not panels.
const MIN_PANEL_PT: f64 = 30.0;

/// Panels next to each other in a row, tops level, with clearly different
/// heights: the row looks ragged.
fn uneven_boxes(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let panels: Vec<&crate::geom::Container> = geometry
        .containers
        .iter()
        .filter(|c| {
            c.visible
                && matches!(c.kind, "block" | "box")
                && c.bbox.height() >= MIN_PANEL_PT
        })
        .collect();
    let mut out = Vec::new();
    for left in &panels {
        for right in &panels {
            let (a, b) = (&left.bbox, &right.bbox);
            let gutter = b.x0 - a.x1;
            if left.page != right.page
                || (a.y0 - b.y0).abs() > 2.0
                || !(0.0..=MAX_ROW_GUTTER_PT).contains(&gutter)
            {
                continue;
            }
            let (low, high) = if a.height() < b.height() {
                (a.height(), b.height())
            } else {
                (b.height(), a.height())
            };
            if low >= MIN_HEIGHT_RATIO * high {
                continue;
            }
            let shorter = if a.height() < b.height() { left } else { right };
            let mut diagnostic = Diagnostic::new(
                UNEVEN_BOXES,
                Severity::Warning,
                format!(
                    "two boxes side by side on page {} are {}pt and {}pt tall, so the row \
                     looks ragged",
                    left.page + 1,
                    round1(a.height()),
                    round1(b.height())
                ),
            );
            diagnostic.loc = diag::resolve_span(world, shorter.span);
            diagnostic.page = Some(left.page + 1);
            diagnostic.bbox = Some(a.union(b).rounded());
            diagnostic.measured = Some(round1(low / high));
            diagnostic.limit = Some(MIN_HEIGHT_RATIO);
            diagnostic.unit = Some("height ratio");
            diagnostic.hints.push(
                "give the boxes the same height: put them in one grid row and let each \
                 fill it (e.g. block(height: 100%) inside cells of a row with a set \
                 height), or stack them"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}
