use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const MIN_SIZE: &str = "L022.text-too-small";
pub const ASPECT: &str = "L023.distorted-image";
pub const RUNT: &str = "L024.runt";
pub const LAST_PAGE: &str = "L025.near-empty-last-page";
pub const SPLIT_FIGURE: &str = "L026.split-figure";
pub const PAGE_FRAME: &str = "L027.inconsistent-text-area";

/// The smallest text size that is still readable in print. Convention.
const MIN_TEXT_PT: f64 = 6.0;

/// How far an image's shown aspect ratio may deviate from its pixels.
const MAX_ASPECT_DEVIATION: f64 = 0.01;

/// A last line narrower than this share of the measure is a runt.
const RUNT_SHARE: f64 = 0.08;

/// A last page whose content covers less than this share of the text area is
/// near-empty.
const MIN_LAST_PAGE_SHARE: f64 = 0.1;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = small_text(world, geometry);
    out.extend(distorted_images(world, geometry));
    out.extend(runts(world, geometry));
    out.extend(last_page(geometry));
    out.extend(split_figures(world, geometry));
    out.extend(page_frames(geometry));
    out
}

fn small_text(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut by_span = BTreeMap::new();
    for item in &geometry.items {
        let Some(size) = item.font_size else { continue };
        if size >= MIN_TEXT_PT
            || item.visible().is_none()
            || in_equation(geometry, item.elem)
        {
            continue;
        }
        by_span.entry(item.span.into_raw().get()).or_insert((item, size));
    }
    by_span
        .into_values()
        .map(|(item, size)| {
            let mut out = Diagnostic::new(
                MIN_SIZE,
                Severity::Warning,
                format!(
                    "text is set at {}pt on page {}, below the readable minimum",
                    round1(size),
                    item.page + 1
                ),
            );
            out.loc = diag::resolve_span(world, item.span);
            out.page = Some(item.page + 1);
            out.bbox = Some(item.bbox.rounded());
            out.measured = Some(round1(size));
            out.limit = Some(MIN_TEXT_PT);
            out.unit = Some("pt");
            out.hints.push(format!("use a text size of at least {MIN_TEXT_PT}pt"));
            out
        })
        .collect()
}

/// Whether an element sits inside an equation, where nested scripts are
/// legitimately small.
fn in_equation(geometry: &Geometry, mut cursor: Option<usize>) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].name == "equation" {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

fn distorted_images(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    geometry
        .items
        .iter()
        .filter(|item| item.kind == ItemKind::Image)
        .filter_map(|item| {
            let [w, h] = item.pixels?;
            let (shown_w, shown_h) = (item.bbox.width(), item.bbox.height());
            if w == 0 || h == 0 || shown_w <= 0.0 || shown_h <= 0.0 {
                return None;
            }
            let natural = f64::from(w) / f64::from(h);
            let shown = shown_w / shown_h;
            let deviation = (shown / natural - 1.0).abs();
            (deviation > MAX_ASPECT_DEVIATION).then_some((item, deviation))
        })
        .map(|(item, deviation)| {
            let mut out = Diagnostic::new(
                ASPECT,
                Severity::Warning,
                format!(
                    "image on page {} is stretched: its aspect ratio differs by {:.0}% \
                     from the source",
                    item.page + 1,
                    deviation * 100.0
                ),
            );
            out.loc = diag::resolve_span(world, item.span);
            out.elem = Some("image".into());
            out.page = Some(item.page + 1);
            out.bbox = Some(item.bbox.rounded());
            out.measured = Some(round1(deviation * 100.0));
            out.limit = Some(MAX_ASPECT_DEVIATION * 100.0);
            out.unit = Some("%");
            out.hints.push(
                "set only the width or only the height, or use fit: \"contain\" or \
                 \"cover\""
                    .into(),
            );
            out
        })
        .collect()
}

fn runts(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let lines = &geometry.lines;
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let Some(elem) = line.elem else { continue };
        if geometry.elements[elem].name != "par" || i == 0 {
            continue;
        }
        let previous = &lines[i - 1];
        let is_last = lines.get(i + 1).is_none_or(|next| next.elem != Some(elem));
        if !is_last || previous.elem != Some(elem) || line.available <= 0.0 {
            continue;
        }
        if line.natural <= 0.0 || line.natural >= RUNT_SHARE * line.available {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            RUNT,
            Severity::Warning,
            format!(
                "the last line of a paragraph on page {} holds only {}pt of text",
                line.page + 1,
                round1(line.natural)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, line.span);
        diagnostic.elem = Some("par".into());
        diagnostic.page = Some(line.page + 1);
        diagnostic.bbox = Some(line.bbox.rounded());
        diagnostic.measured = Some(round1(line.natural));
        diagnostic.limit = Some(round1(RUNT_SHARE * line.available));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "rephrase the paragraph slightly so its last line holds more than a word \
             fragment"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn last_page(geometry: &Geometry) -> Vec<Diagnostic> {
    if geometry.pages.len() < 2 {
        return Vec::new();
    }
    let page = geometry.pages.len() - 1;
    let continued = geometry
        .regions
        .iter()
        .any(|r| r.page + 1 == page && r.kind == "page-body" && !r.last_in_run);
    if !continued {
        return Vec::new();
    }
    let Some((index, region)) = geometry
        .regions
        .iter()
        .enumerate()
        .find(|(_, r)| r.page == page && r.kind == "page-body")
    else {
        return Vec::new();
    };
    let content = geometry
        .items
        .iter()
        .filter(|item| item.region == Some(index) && item.kind != ItemKind::Link)
        .filter_map(|item| item.visible())
        .reduce(|a, b| a.union(&b));
    let Some(content) = content else { return Vec::new() };
    let share = (content.y1 - region.bbox.y0) / region.bbox.height();
    if share >= MIN_LAST_PAGE_SHARE {
        return Vec::new();
    }
    let mut out = Diagnostic::new(
        LAST_PAGE,
        Severity::Warning,
        format!("the last page holds only {:.0}% of a page of content", share * 100.0),
    );
    out.page = Some(page + 1);
    out.bbox = Some(content.rounded());
    out.measured = Some(round1(share * 100.0));
    out.limit = Some(MIN_LAST_PAGE_SHARE * 100.0);
    out.unit = Some("%");
    out.hints.push(
        "tighten the preceding content slightly so it fits on the previous page, or \
         add closing content"
            .into(),
    );
    vec![out]
}

fn split_figures(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    geometry
        .elements
        .iter()
        .filter(|e| e.name == "figure" && e.boxes.len() > 1)
        .map(|figure| {
            let pages: Vec<String> =
                figure.boxes.iter().map(|b| (b.page + 1).to_string()).collect();
            let mut out = Diagnostic::new(
                SPLIT_FIGURE,
                Severity::Warning,
                format!("figure is split across pages {}", pages.join(", ")),
            );
            out.loc = diag::resolve_span(world, figure.span);
            out.elem = Some("figure".into());
            out.label = figure.label.clone();
            out.page = figure.boxes.first().map(|b| b.page + 1);
            out.hints.push(
                "keep the figure on one page: reduce its size, or let a long table \
                 break outside a figure"
                    .into(),
            );
            out
        })
        .collect()
}

fn page_frames(geometry: &Geometry) -> Vec<Diagnostic> {
    let bodies: Vec<_> =
        geometry.regions.iter().filter(|r| r.kind == "page-body").collect();
    let Some(first) = bodies.first() else { return Vec::new() };
    let mut out = Vec::new();
    for body in &bodies[1..] {
        let page = &geometry.pages[body.page];
        let reference = &geometry.pages[first.page];
        if (page.width - reference.width).abs() > 0.5
            || (page.height - reference.height).abs() > 0.5
        {
            continue;
        }
        let mirrored_x0 = reference.width - first.bbox.x1;
        let same_x = (body.bbox.x0 - first.bbox.x0).abs() <= 0.5
            || (body.bbox.x0 - mirrored_x0).abs() <= 0.5;
        let same_y = (body.bbox.y0 - first.bbox.y0).abs() <= 0.5;
        let same_size = (body.bbox.width() - first.bbox.width()).abs() <= 0.5
            && (body.bbox.height() - first.bbox.height()).abs() <= 0.5;
        if same_x && same_y && same_size {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            PAGE_FRAME,
            Severity::Warning,
            format!(
                "the text area of page {} differs from page {} ({:.1}x{:.1}pt at \
                 {:.1},{:.1} vs {:.1}x{:.1}pt at {:.1},{:.1})",
                body.page + 1,
                first.page + 1,
                body.bbox.width(),
                body.bbox.height(),
                body.bbox.x0,
                body.bbox.y0,
                first.bbox.width(),
                first.bbox.height(),
                first.bbox.x0,
                first.bbox.y0
            ),
        );
        diagnostic.page = Some(body.page + 1);
        diagnostic.bbox = Some(body.bbox.rounded());
        diagnostic.hints.push(
            "set the page margins once at the start of the document instead of \
             changing them mid-document"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}
