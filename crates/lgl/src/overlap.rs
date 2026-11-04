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

/// Glyphs of an inline equation are not kerned against text or another
/// equation, so a smaller overlap is already a collision: stacked fractions
/// on neighbouring lines run into each other by only a couple of points.
const MATH_SLACK_PT: f64 = 0.5;

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
    out.extend(boxes_over_text(world, geometry));
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
        (Some(_), Some(_)) if a.equation.is_some() || b.equation.is_some() => {
            MATH_SLACK_PT
        }
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

pub const BOX_OVER_TEXT: &str = "L080.box-over-text";

/// Shapes that belong to the text they cover.
const DECORATIONS: [&str; 4] = ["underline", "strike", "overline", "highlight"];

/// A filled or framed box (at least this tall; thinner shapes are lines,
/// see L050) painted over part of text outside it.
const MIN_BOX_PT: f64 = 3.0;

/// A box painted over part of text it does not belong to: blocks that ran
/// into each other (often a v(1fr) that shrank to nothing on a full page,
/// which also removes the normal gap between the blocks).
pub fn boxes_over_text(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let texts: Vec<(usize, &Item, BBox)> = geometry
        .items
        .iter()
        .enumerate()
        .filter(|(_, i)| {
            i.kind == ItemKind::Text
                && i.font_size.is_some()
                && i.text.as_deref().is_some_and(|t| !t.trim().is_empty())
        })
        .filter_map(|(index, i)| i.visible().map(|b| (index, i, b)))
        .collect();
    let mut out = Vec::new();
    for (index, shape) in geometry.items.iter().enumerate() {
        if shape.kind != ItemKind::Shape {
            continue;
        }
        let Some(s) = shape.visible() else { continue };
        if s.width() <= 0.0 || s.height() < MIN_BOX_PT {
            continue;
        }
        // Only a box painted after the text covers it; one painted before is
        // a background.
        let hit = texts.iter().find_map(|&(at, text, t)| {
            if at > index || text.page != shape.page || belongs_to(geometry, shape, text)
            {
                return None;
            }
            let o = s.intersect(&t)?;
            let inside = s.x0 <= t.x0 + 0.5
                && s.y0 <= t.y0 + 0.5
                && s.x1 >= t.x1 - 0.5
                && s.y1 >= t.y1 - 0.5;
            (o.width() > 1.0 && o.height() > 1.0 && !inside).then_some((text, o))
        });
        let Some((text, o)) = hit else { continue };
        let words = text.text.as_deref().unwrap_or_default().trim();
        let mut diagnostic = Diagnostic::new(
            BOX_OVER_TEXT,
            Severity::Error,
            format!(
                "a box on page {} is painted over the text {words:?}, covering {}pt of it",
                shape.page + 1,
                round1(o.height())
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(shape.span, shape.elem));
        diagnostic.page = Some(shape.page + 1);
        diagnostic.bbox = Some(o.rounded());
        diagnostic.measured = Some(round1(o.height()));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "leave space before the box. A v(1fr) before it shrinks to nothing on a full \
             page and also removes the normal gap between blocks, so add a fixed gap such as \
             v(1em) next to it"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Whether the shape belongs to the text: a decoration of it, or the fill or
/// border of a container the text sits in.
fn belongs_to(geometry: &Geometry, shape: &Item, text: &Item) -> bool {
    if let Some(e) = shape.elem
        && text.elem == Some(e)
        && DECORATIONS.contains(&geometry.elements[e].name)
    {
        return true;
    }
    let Some(own) = shape.container else { return false };
    let mut cursor = text.container;
    while let Some(c) = cursor {
        if c == own {
            return true;
        }
        cursor = geometry.containers[c].parent;
    }
    false
}
