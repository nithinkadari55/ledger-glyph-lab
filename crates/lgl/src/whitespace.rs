use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const PAGE_UNDERFILL: &str = "L007.page-underfill";
pub const NEAR_EMPTY_PAGE: &str = "L078.near-empty-page";

/// A page inside the document (not the last) whose content ends within this
/// share of its text area holds only a sliver, typically content that spilled
/// over before a page break. Measured on 168 generated documents: every
/// inner page uses at least 35% of its text area except one at 4%.
const MIN_INNER_PAGE_USE: f64 = 0.25;

/// A page whose content reaches this share of its text area is full.
const FULL_PAGE_USE: f64 = 0.9;

/// The share of the text area that may stay empty at the bottom of a page that
/// was ended for lack of space. Convention, tunable.
const MAX_EMPTY_SHARE: f64 = 0.2;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, region) in geometry.regions.iter().enumerate() {
        if region.last_in_run || region.kind != "page-body" {
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
    out.extend(near_empty_pages(world, geometry));
    out
}

/// An inner page that holds only a sliver of content spilled over from a
/// full page before it, even when a page break follows it (the break is
/// meant to start a new page, not to leave one almost blank).
fn near_empty_pages(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let last = geometry.pages.len().saturating_sub(1);
    let mut out = Vec::new();
    for (index, region) in geometry.regions.iter().enumerate() {
        if region.kind != "page-body" || region.page >= last {
            continue;
        }
        let Some(bottom) = content_bottom(geometry, index) else { continue };
        let used = (bottom - region.bbox.y0) / region.bbox.height();
        if used >= MIN_INNER_PAGE_USE {
            continue;
        }
        // Only a spill: the page before is full. A short page that ends on
        // purpose (a cover, a section end) is followed by a page break too.
        let previous = geometry.regions.iter().enumerate().find(|(_, r)| {
            r.kind == "page-body" && region.page > 0 && r.page == region.page - 1
        });
        let Some((before, previous)) = previous else { continue };
        let full = content_bottom(geometry, before).is_some_and(|b| {
            (b - previous.bbox.y0) / previous.bbox.height() >= FULL_PAGE_USE
        });
        if !full {
            continue;
        }
        let first = first_item(geometry, region.page).map(|i| &geometry.items[i]);
        let mut diagnostic = Diagnostic::new(
            NEAR_EMPTY_PAGE,
            Severity::Warning,
            format!(
                "page {} uses only {}% of its text area and the document continues on the \
                 next page, so it is almost blank",
                region.page + 1,
                (used * 100.0).round()
            ),
        );
        if let Some(item) = first {
            diagnostic.loc = diag::resolve_span(world, geometry.span_for(item.span, item.elem));
        }
        diagnostic.page = Some(region.page + 1);
        diagnostic.bbox = Some(region.bbox.rounded());
        diagnostic.measured = Some(round1(used * 100.0));
        diagnostic.limit = Some(MIN_INNER_PAGE_USE * 100.0);
        diagnostic.unit = Some("% of the text area");
        diagnostic.hints.push(
            "the content before a page break spilled onto this page: shorten or tighten the \
             previous page so it fits, or remove the page break"
                .into(),
        );
        out.push(diagnostic);
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
            item.page == page
                && geometry.is_body(item.region)
                && item.kind != ItemKind::Link
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
        let span = blocker
            .map(|c| c.span)
            .filter(|s| !s.is_detached())
            .unwrap_or(item.span);
        out.anchor = blocker.map(|c| c.span);
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
