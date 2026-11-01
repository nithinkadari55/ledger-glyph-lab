use std::collections::BTreeMap;

use typst_layout::PagedDocument;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item};
use crate::world::LabWorld;

pub const PAGE_OVERFLOW: &str = "L001.page-overflow";

const TOLERANCE_PT: f64 = 0.5;

pub fn run(world: &LabWorld, _document: &PagedDocument, geometry: &Geometry) -> Vec<Diagnostic> {
    page_overflow(world, geometry)
}

fn visible(item: &Item) -> Option<BBox> {
    match item.clip {
        Some(clip) => item.bbox.intersect(&clip),
        None => Some(item.bbox),
    }
}

struct Overflow {
    amount: f64,
    side: &'static str,
    item: usize,
}

fn page_overflow(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut worst: BTreeMap<(usize, Option<usize>, Option<usize>), Overflow> = BTreeMap::new();

    for (index, item) in geometry.items.iter().enumerate() {
        let Some(bbox) = visible(item) else { continue };
        let page = &geometry.pages[item.page];
        let [bleed_left, bleed_top, bleed_right, bleed_bottom] = page.bleed;
        let sides = [
            (-bleed_left - bbox.x0, "left"),
            (-bleed_top - bbox.y0, "top"),
            (bbox.x1 - (page.width + bleed_right), "right"),
            (bbox.y1 - (page.height + bleed_bottom), "bottom"),
        ];
        let Some(&(amount, side)) =
            sides.iter().max_by(|a, b| a.0.total_cmp(&b.0))
        else {
            continue;
        };
        if amount <= TOLERANCE_PT {
            continue;
        }
        let key = (item.page, item.elem, item.elem.is_none().then_some(index));
        let entry = worst.entry(key).or_insert(Overflow { amount, side, item: index });
        if amount > entry.amount {
            *entry = Overflow { amount, side, item: index };
        }
    }

    worst
        .into_values()
        .map(|overflow| {
            let item = &geometry.items[overflow.item];
            let element = item.elem.map(|e| &geometry.elements[e]);
            let mut out = Diagnostic::new(
                PAGE_OVERFLOW,
                Severity::Error,
                format!(
                    "content extends {:.1}pt beyond the {} edge of page {}",
                    overflow.amount,
                    overflow.side,
                    item.page + 1
                ),
            );
            out.loc = diag::resolve_span(world, item.span)
                .or_else(|| element.and_then(|e| diag::resolve_span(world, e.span)));
            out.elem = element.map(|e| e.name.to_string());
            out.label = element.and_then(|e| e.label.clone());
            out.page = Some(item.page + 1);
            out.bbox = Some(item.bbox);
            out.measured = Some(round(overflow.amount));
            out.limit = Some(TOLERANCE_PT);
            out.unit = Some("pt");
            out.hints.push(match overflow.side {
                "left" | "right" => {
                    "reduce the width: use relative (%) or fractional (fr) sizes, enable hyphenation, or allow the content to wrap".into()
                }
                _ => "allow the content to break across pages, or reduce its height".into(),
            });
            out
        })
        .collect()
}

fn round(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
