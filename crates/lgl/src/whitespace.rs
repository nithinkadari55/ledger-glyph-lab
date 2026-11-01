use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const PAGE_UNDERFILL: &str = "L007.page-underfill";

/// The share of the text area that may stay empty at the bottom of a page that
/// was ended for lack of space. Convention, tunable.
const MAX_EMPTY_SHARE: f64 = 0.2;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, region) in geometry.regions.iter().enumerate() {
        if region.last_in_run {
            continue;
        }
        let Some(bottom) = content_bottom(geometry, index) else { continue };
        let empty = region.bbox.y1 - bottom;
        let limit = MAX_EMPTY_SHARE * region.bbox.height();
        if empty <= limit {
            continue;
        }
        out.push(underfill(world, geometry, region.page, empty, limit, &region.bbox));
    }
    out
}

/// The lowest point of visible content in a region.
fn content_bottom(geometry: &Geometry, region: usize) -> Option<f64> {
    geometry
        .items
        .iter()
        .filter(|item| item.region == Some(region) && item.kind != ItemKind::Link)
        .filter_map(|item| item.visible())
        .map(|bbox| bbox.y1)
        .max_by(f64::total_cmp)
}

/// The first visible item in the body of the given page.
fn first_item(geometry: &Geometry, page: usize) -> Option<usize> {
    geometry
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            item.page == page && item.region.is_some() && item.kind != ItemKind::Link
        })
        .min_by(|(_, a), (_, b)| a.bbox.y0.total_cmp(&b.bbox.y0))
        .map(|(index, _)| index)
}

fn underfill(
    world: &LabWorld,
    geometry: &Geometry,
    page: usize,
    empty: f64,
    limit: f64,
    body: &BBox,
) -> Diagnostic {
    let next = first_item(geometry, page + 1).map(|i| &geometry.items[i]);
    let blocker = next.and_then(|item| {
        let mut cursor = item.container;
        let mut found = None;
        while let Some(index) = cursor {
            let container = &geometry.containers[index];
            if container.page == page + 1 {
                found = Some(container);
            }
            cursor = container.parent;
        }
        found
    });

    let empty = round1(empty);
    let mut out = Diagnostic::new(
        PAGE_UNDERFILL,
        Severity::Warning,
        format!(
            "page {} leaves {empty}pt ({:.0}% of the text area) empty at the bottom \
             because the following content does not fit",
            page + 1,
            100.0 * empty / body.height()
        ),
    );
    out.page = Some(page + 1);
    out.bbox = Some(BBox { y0: body.y1 - empty, ..*body }.rounded());
    out.measured = Some(empty);
    out.limit = Some(round1(limit));
    out.unit = Some("pt");
    if let Some(item) = next {
        let span = blocker.map(|c| c.span).filter(|s| !s.is_detached()).unwrap_or(item.span);
        out.loc = diag::resolve_span(world, span);
        out.elem = blocker
            .map(|c| c.kind.to_string())
            .or_else(|| item.elem.map(|e| geometry.elements[e].name.to_string()));
        if let Some(container) = blocker {
            out.hints.push(format!(
                "the {} starting page {} is {:.1}pt tall",
                container.kind,
                page + 2,
                container.bbox.height()
            ));
        }
    }
    out.hints.push(
        "let the content break across pages (breakable: true, allow table rows to \
         split), make figures float (placement: auto), or reduce its height"
            .into(),
    );
    out
}
