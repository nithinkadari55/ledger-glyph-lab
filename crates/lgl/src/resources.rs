use std::collections::BTreeMap;

use typst::syntax::Span;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, round1};
use crate::world::LabWorld;

pub const MISSING_GLYPH: &str = "L008.missing-glyph";
pub const IMAGE_RESOLUTION: &str = "L009.image-resolution";

/// The effective resolution below which raster images look blurry in print.
/// Convention, tunable.
const MIN_DPI: f64 = 150.0;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = missing_glyphs(world, geometry);
    out.extend(image_resolution(world, geometry));
    out
}

fn missing_glyphs(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut by_span: BTreeMap<(usize, u64), (Span, String)> = BTreeMap::new();
    for item in &geometry.items {
        let Some(missing) = &item.missing else { continue };
        let entry = by_span
            .entry((item.page, item.span.into_raw().get()))
            .or_insert_with(|| (item.span, String::new()));
        for c in missing.chars() {
            if !entry.1.contains(c) {
                entry.1.push(c);
            }
        }
    }

    by_span
        .into_iter()
        .map(|((page, _), (span, chars))| {
            let codes: Vec<String> =
                chars.chars().map(|c| format!("U+{:04X}", c as u32)).collect();
            let mut out = Diagnostic::new(
                MISSING_GLYPH,
                Severity::Error,
                format!(
                    "no available font can display {chars:?} ({}) on page {}; it renders \
                     as an empty box",
                    codes.join(", "),
                    page + 1
                ),
            );
            out.loc = diag::resolve_span(world, span);
            out.page = Some(page + 1);
            out.hints.push(
                "add a fallback font that covers these characters to the text font list, \
                 or replace the characters"
                    .into(),
            );
            out
        })
        .collect()
}

fn image_resolution(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    geometry
        .items
        .iter()
        .filter_map(|item| {
            let [px_w, px_h] = item.pixels?;
            let (w, h) = (item.bbox.width(), item.bbox.height());
            if w <= 0.0 || h <= 0.0 {
                return None;
            }
            let dpi = (f64::from(px_w) / (w / 72.0)).min(f64::from(px_h) / (h / 72.0));
            (dpi < MIN_DPI).then_some((item, dpi))
        })
        .map(|(item, dpi)| {
            let page = item.page + 1;
            let mut out = Diagnostic::new(
                IMAGE_RESOLUTION,
                Severity::Warning,
                format!(
                    "image is shown at {:.0} dpi on page {page} ({}x{} px over {:.1}pt x \
                     {:.1}pt) and will look blurry in print",
                    dpi,
                    item.pixels.unwrap()[0],
                    item.pixels.unwrap()[1],
                    item.bbox.width(),
                    item.bbox.height()
                ),
            );
            out.loc = diag::resolve_span(world, item.span);
            out.elem = Some("image".into());
            out.page = Some(page);
            out.bbox = Some(item.bbox.rounded());
            out.measured = Some(round1(dpi));
            out.limit = Some(MIN_DPI);
            out.unit = Some("dpi");
            out.hints.push(format!(
                "show the image smaller (at most {:.1}pt wide) or use a higher-resolution \
                 source",
                f64::from(item.pixels.unwrap()[0]) / MIN_DPI * 72.0
            ));
            out
        })
        .collect()
}
