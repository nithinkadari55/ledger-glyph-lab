use std::collections::BTreeSet;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const LOW_CONTRAST: &str = "L047.low-contrast";

/// The lowest contrast ratio at which text stays legible: the WCAG minimum
/// for large text. Below `INVISIBLE_RATIO` the text all but disappears.
const MIN_RATIO: f64 = 3.0;
const INVISIBLE_RATIO: f64 = 1.5;

/// A fill counts as the background of text when it covers this share of the
/// text's box.
const MIN_COVER: f64 = 0.5;

const WHITE: [f64; 3] = [1.0, 1.0, 1.0];

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for (index, item) in geometry.items.iter().enumerate() {
        if item.kind != ItemKind::Text
            || item
                .text
                .as_deref()
                .is_none_or(|t| !t.chars().any(char::is_alphanumeric))
        {
            continue;
        }
        // Watermarks and other page decorations are faint on purpose.
        if item.region.is_some_and(|r| {
            matches!(geometry.regions[r].kind, "background" | "foreground")
        }) || overlay(geometry, item.elem)
        {
            continue;
        }
        let (Some(color), Some(bbox)) = (item.color, item.visible()) else { continue };
        let Some(background) = background(geometry, index) else { continue };
        let foreground = blend(color, background);
        let ratio = contrast(foreground, background);
        if ratio >= MIN_RATIO || !seen.insert((item.page, item.span.into_raw().get())) {
            continue;
        }
        let severity =
            if ratio < INVISIBLE_RATIO { Severity::Error } else { Severity::Warning };
        let mut diagnostic = Diagnostic::new(
            LOW_CONTRAST,
            severity,
            format!(
                "text \"{}\" on page {} has a contrast ratio of {}:1 ({} on {}), below \
                 {MIN_RATIO}:1",
                excerpt(item.text.as_deref()),
                item.page + 1,
                round1(ratio),
                hex(foreground),
                hex(background)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, item.span);
        diagnostic.page = Some(item.page + 1);
        diagnostic.bbox = Some(bbox.rounded());
        diagnostic.measured = Some(round1(ratio));
        diagnostic.limit = Some(MIN_RATIO);
        diagnostic.unit = Some("ratio");
        diagnostic.hints.push(
            "darken the text or lighten its background (or the reverse) until they \
             differ clearly in lightness"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Whether the content is labelled as an intended overlay.
fn overlay(geometry: &Geometry, mut cursor: Option<usize>) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].label.as_deref()
            == Some(crate::overlap::ALLOW_OVERLAP)
        {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// The colour painted beneath a text item: the topmost solid fill covering
/// it, or the white page. `None` when the text sits on an image or a gradient,
/// whose colour under the text is unknown.
fn background(geometry: &Geometry, text: usize) -> Option<[f64; 3]> {
    let item = &geometry.items[text];
    let bbox = item.visible()?;
    let area = bbox.width() * bbox.height();
    for below in geometry.items[..text].iter().rev() {
        if below.page != item.page || below.kind == ItemKind::Link {
            continue;
        }
        if below.kind != ItemKind::Image && below.color.is_none() && !below.patterned {
            continue;
        }
        let Some(cover) = below.visible().and_then(|b| b.intersect(&bbox)) else {
            continue;
        };
        if cover.width() * cover.height() < MIN_COVER * area {
            continue;
        }
        if below.kind == ItemKind::Image || below.patterned {
            return None;
        }
        let Some(color) = below.color else { continue };
        if below.kind == ItemKind::Text || color[3] < 0.5 {
            continue;
        }
        return Some(blend(color, WHITE));
    }
    Some(WHITE)
}

fn blend(color: [f32; 4], under: [f64; 3]) -> [f64; 3] {
    let alpha = f64::from(color[3]);
    std::array::from_fn(|i| alpha * f64::from(color[i]) + (1.0 - alpha) * under[i])
}

fn luminance(rgb: [f64; 3]) -> f64 {
    let linear = |c: f64| {
        if c <= 0.04045 { c / 12.92 } else { libm::pow((c + 0.055) / 1.055, 2.4) }
    };
    0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
}

fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn hex(rgb: [f64; 3]) -> String {
    let byte = |c: f64| (c.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", byte(rgb[0]), byte(rgb[1]), byte(rgb[2]))
}

fn excerpt(text: Option<&str>) -> String {
    let text = text.unwrap_or_default().trim();
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(24).collect();
    if chars.next().is_some() { format!("{head}…") } else { head }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contrast_matches_wcag() {
        assert!((contrast([0.0; 3], WHITE) - 21.0).abs() < 0.01);
        assert!((contrast(WHITE, WHITE) - 1.0).abs() < 1e-9);
        // #767676 on white is the classic 4.54:1 grey.
        let grey = 118.0 / 255.0;
        assert!((contrast([grey; 3], WHITE) - 4.54).abs() < 0.01);
    }
}
