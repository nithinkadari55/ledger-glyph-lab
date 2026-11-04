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
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(item.span, item.elem));
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

pub const PAGE_LABEL: &str = "L076.wrong-page-label";

/// A "Page N of M" label in a header or footer whose numbers do not match the
/// page it is on or the document's page count (often a typed-in total).
pub fn page_labels(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let total = geometry.pages.len();
    let mut out = Vec::new();
    for page in 0..total {
        for kind in ["header", "footer"] {
            let mut items: Vec<&crate::geom::Item> = geometry
                .items
                .iter()
                .filter(|i| {
                    i.page == page
                        && i.kind == ItemKind::Text
                        && i.region.is_some_and(|r| geometry.regions[r].kind == kind)
                })
                .collect();
            items.sort_by(|a, b| {
                (a.bbox.y0.round(), a.bbox.x0)
                    .partial_cmp(&(b.bbox.y0.round(), b.bbox.x0))
                    .unwrap()
            });
            let text: String = items
                .iter()
                .filter_map(|i| i.text.as_deref())
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(" ");
            for (number, of) in page_of_pairs(&text) {
                if number == page + 1 && of == total {
                    continue;
                }
                let Some(first) = items.first() else { continue };
                let mut diagnostic = Diagnostic::new(
                    PAGE_LABEL,
                    Severity::Error,
                    format!(
                        "the {kind} on page {} reads \"{number} of {of}\", but this is page {} \
                         of {total}",
                        page + 1,
                        page + 1
                    ),
                );
                diagnostic.loc =
                    diag::resolve_span(world, geometry.span_for(first.span, first.elem));
                diagnostic.page = Some(page + 1);
                diagnostic.hints.push(
                    "number pages with the counter instead of typed numbers: \
                     context counter(page).display(\"1 of 1\", both: true)"
                        .into(),
                );
                out.push(diagnostic);
            }
        }
    }
    out
}

/// The "N of M" and "N / M" pairs in a text.
fn page_of_pairs(text: &str) -> Vec<(usize, usize)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = Vec::new();
    for (k, word) in words.iter().enumerate() {
        // "3/4" written without spaces.
        if let Some((a, b)) = word.split_once('/')
            && let (Ok(a), Ok(b)) = (a.parse::<usize>(), b.parse::<usize>())
        {
            out.push((a, b));
            continue;
        }
        if k + 2 < words.len()
            && (words[k + 1] == "of" || words[k + 1] == "/")
            && let (Ok(a), Ok(b)) = (word.parse::<usize>(), words[k + 2].parse::<usize>())
        {
            out.push((a, b));
        }
    }
    out
}
