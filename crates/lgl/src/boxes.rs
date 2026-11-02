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

/// Text in neighbouring cells must be separated by at least this fraction of
/// the font size (about a word space).
const MIN_CELL_GAP_EM: f64 = 0.2;

/// Text runs overlapping by more than this fraction of the font size are
/// reported by L006 instead.
const OVERLAP_SLACK_EM: f64 = 0.25;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = hollow_boxes(world, geometry);
    out.extend(touching_cells(world, geometry));
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
            if item.kind != ItemKind::Shape || !is_frame(&visible, &container.bbox) {
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

    let mut seen = BTreeMap::new();
    for texts in pages.values() {
        for left in texts {
            for right in texts {
                if left.cell == right.cell {
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
            out.loc = diag::resolve_span(world, b.span);
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
