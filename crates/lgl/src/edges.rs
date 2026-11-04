//! Content that printers may cut off: anything closer to the page edge than
//! the safe zone, unless it deliberately runs off the page (bleed).

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const PRINT_EDGE: &str = "L067.print-edge";

/// 5 mm. Desktop printers cannot print the outer 3 to 6 mm of a sheet, and
/// print shops ask for a safe zone of 3 to 5 mm inside the trim.
const SAFE_PT: f64 = 14.17;

/// Content within this distance of an edge runs off the page on purpose.
const BLEED_PT: f64 = 0.5;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, page) in geometry.pages.iter().enumerate() {
        let closest = geometry
            .items
            .iter()
            .filter(|item| item.page == index && item.kind != ItemKind::Link)
            .filter(|item| {
                item.kind != ItemKind::Text
                    || item.text.as_deref().is_some_and(|t| !t.trim().is_empty())
            })
            .filter_map(|item| {
                let b = item.visible()?;
                let distance =
                    b.x0.min(b.y0).min(page.width - b.x1).min(page.height - b.y1);
                (distance > BLEED_PT && distance < SAFE_PT).then_some((item, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        let Some((item, distance)) = closest else { continue };
        let what = match item.kind {
            ItemKind::Text => "text",
            ItemKind::Image => "an image",
            _ => "a line or shape",
        };
        let mut diagnostic = Diagnostic::new(
            PRINT_EDGE,
            Severity::Warning,
            format!(
                "{what} on page {} comes within {}pt of the page edge; printers may cut \
                 off anything closer than 5 mm (14pt)",
                index + 1,
                round1(distance)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, geometry.span_for(item.span, item.elem));
        diagnostic.page = Some(index + 1);
        diagnostic.bbox = item.visible().map(|b| b.rounded());
        diagnostic.measured = Some(round1(distance));
        diagnostic.limit = Some(round1(SAFE_PT));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "keep frames, footers and other content at least 5 mm inside the page edge \
             (larger page margins, or a shorter footer); backgrounds and bands that \
             should touch the edge must run off the page"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}
