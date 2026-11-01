use std::collections::HashMap;

use typst::syntax::Span;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const OVERLAP: &str = "L006.overlap";

/// The label that marks content whose overlap with other content is intended,
/// e.g. a watermark.
pub const ALLOW_OVERLAP: &str = "allow-overlap";

/// Overlaps between text runs smaller than this fraction of the font size in
/// either direction are kerning, italic overhang or tight leading.
const TEXT_SLACK_EM: f64 = 0.25;

/// Overlaps involving images smaller than this in either direction are ignored.
const IMAGE_SLACK_PT: f64 = 2.0;

struct Candidate<'a> {
    index: usize,
    item: &'a Item,
    bbox: BBox,
    equation: Option<usize>,
}

struct Finding {
    page: usize,
    width: f64,
    height: f64,
    upper: usize,
    lower: usize,
}

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut pages: Vec<Vec<Candidate>> =
        (0..geometry.pages.len()).map(|_| Vec::new()).collect();
    for (index, item) in geometry.items.iter().enumerate() {
        if !matches!(item.kind, ItemKind::Text | ItemKind::Image) {
            continue;
        }
        let Some(bbox) = item.visible() else { continue };
        if bbox.width() <= 0.0 || bbox.height() <= 0.0 || allowed(geometry, item.elem) {
            continue;
        }
        let equation = ancestor_named(geometry, item.elem, "equation");
        pages[item.page].push(Candidate { index, item, bbox, equation });
    }

    let mut findings: HashMap<(Span, Span), Finding> = HashMap::new();
    for candidates in &mut pages {
        candidates.sort_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
        let mut active: Vec<usize> = Vec::new();
        for current in 0..candidates.len() {
            let top = candidates[current].bbox.y0;
            active.retain(|&other| candidates[other].bbox.y1 > top);
            for &other in &active {
                let (a, b) = (&candidates[other], &candidates[current]);
                let Some((width, height)) = conflict(a, b) else { continue };
                let (lower, upper) = if a.index < b.index {
                    (a.index, b.index)
                } else {
                    (b.index, a.index)
                };
                let spans = (geometry.items[lower].span, geometry.items[upper].span);
                let entry = findings.entry(spans).or_insert(Finding {
                    page: a.item.page,
                    width,
                    height,
                    upper,
                    lower,
                });
                if width * height > entry.width * entry.height {
                    entry.width = width;
                    entry.height = height;
                    entry.upper = upper;
                    entry.lower = lower;
                }
            }
            active.push(current);
        }
    }

    let mut out: Vec<Diagnostic> = findings
        .into_values()
        .map(|f| to_diagnostic(world, geometry, f))
        .collect();
    out.sort_by(|a, b| a.page.cmp(&b.page).then_with(|| a.message.cmp(&b.message)));
    out
}

/// The size of the overlap between two candidates, if it is a conflict.
fn conflict(a: &Candidate, b: &Candidate) -> Option<(f64, f64)> {
    if a.equation.is_some() && a.equation == b.equation {
        return None;
    }
    let overlap = a.bbox.intersect(&b.bbox)?;
    let (width, height) = (overlap.width(), overlap.height());
    let slack = match (a.item.font_size, b.item.font_size) {
        (Some(x), Some(y)) => TEXT_SLACK_EM * x.min(y),
        _ => IMAGE_SLACK_PT,
    };
    (width > slack && height > slack).then_some((width, height))
}

fn allowed(geometry: &Geometry, mut cursor: Option<usize>) -> bool {
    while let Some(index) = cursor {
        let element = &geometry.elements[index];
        if element.label.as_deref() == Some(ALLOW_OVERLAP) {
            return true;
        }
        cursor = element.parent;
    }
    false
}

fn ancestor_named(
    geometry: &Geometry,
    mut cursor: Option<usize>,
    name: &str,
) -> Option<usize> {
    while let Some(index) = cursor {
        if geometry.elements[index].name == name {
            return Some(index);
        }
        cursor = geometry.elements[index].parent;
    }
    None
}

fn describe(item: &Item) -> &'static str {
    match item.kind {
        ItemKind::Image => "image",
        _ => "text",
    }
}

fn to_diagnostic(world: &LabWorld, geometry: &Geometry, finding: Finding) -> Diagnostic {
    let upper = &geometry.items[finding.upper];
    let lower = &geometry.items[finding.lower];
    let page = finding.page + 1;
    let (width, height) = (round1(finding.width), round1(finding.height));
    let lower_loc = diag::resolve_span(world, lower.span);
    let mut out = Diagnostic::new(
        OVERLAP,
        Severity::Error,
        format!(
            "{} overlaps {} by {width}pt x {height}pt on page {page}",
            describe(upper),
            describe(lower),
        ),
    );
    out.loc = diag::resolve_span(world, upper.span).or_else(|| lower_loc.clone());
    out.elem = upper.elem.map(|e| geometry.elements[e].name.to_string());
    out.page = Some(page);
    out.bbox = upper.visible().map(|b| b.rounded());
    out.measured = Some(width.min(height));
    out.limit = Some(0.0);
    out.unit = Some("pt");
    if let Some(loc) = lower_loc {
        out.hints.push(format!(
            "the other content is at {}:{}:{}",
            loc.file, loc.line, loc.col
        ));
    }
    out.hints.push(format!(
        "keep both in the normal flow instead of using place, move or negative spacing; \
         label intended overlays with <{ALLOW_OVERLAP}>"
    ));
    out
}
