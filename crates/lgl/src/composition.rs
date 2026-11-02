use std::collections::BTreeSet;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, ItemKind, Line, round1};
use crate::world::LabWorld;

pub const CENTERED_PARAGRAPH: &str = "L044.centered-paragraph";
pub const FIGURE_SPACING: &str = "L045.figure-too-close";

/// A measure at least this share of the text area is wide; centred text that
/// wraps across it is hard to read. Convention.
const WIDE_MEASURE_SHARE: f64 = 0.6;

/// A line this full was broken by wrapping, not by a deliberate line break.
const WRAPPED_SHARE: f64 = 0.85;

/// A centred line leaves at least this share of the measure on each side.
const CENTERED_SIDE_SHARE: f64 = 0.1;

/// Display text (titles) may be centred over several lines.
const DISPLAY_TEXT_PT: f64 = 14.0;

/// A figure must be set off from the surrounding text by at least this much
/// more than the text's own line pitch, or its caption reads as part of the
/// paragraph.
const MIN_FIGURE_EXTRA_EM: f64 = 0.3;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = centered_paragraphs(world, geometry);
    out.extend(figure_spacing(world, geometry));
    out
}

struct Ink {
    x0: f64,
    x1: f64,
    size: f64,
}

/// The horizontal extent and largest size of the text on a line.
fn ink(geometry: &Geometry, line: &Line) -> Option<Ink> {
    let b = &line.bbox;
    let mut out: Option<Ink> = None;
    for item in &geometry.items {
        if item.page != line.page || item.kind != ItemKind::Text {
            continue;
        }
        let Some(size) = item.font_size else { continue };
        let cx = (item.bbox.x0 + item.bbox.x1) / 2.0;
        let cy = (item.bbox.y0 + item.bbox.y1) / 2.0;
        if cy < b.y0 || cy > b.y1 || cx < b.x0 - 1.0 || cx > b.x1 + 1.0 {
            continue;
        }
        out = Some(match out {
            Some(ink) => Ink {
                x0: ink.x0.min(item.bbox.x0),
                x1: ink.x1.max(item.bbox.x1),
                size: ink.size.max(size),
            },
            None => Ink { x0: item.bbox.x0, x1: item.bbox.x1, size },
        });
    }
    out
}

fn body_width(geometry: &Geometry, page: usize) -> Option<f64> {
    geometry
        .regions
        .iter()
        .find(|r| r.kind == "page-body" && r.page == page)
        .map(|r| r.bbox.width())
}

fn centered_paragraphs(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for pair in geometry.lines.windows(2) {
        let (above, line) = (&pair[0], &pair[1]);
        if above.page != line.page
            || above.container != line.container
            || !geometry.is_body(line.region)
            || (above.bbox.x0 - line.bbox.x0).abs() > 0.5
            || (above.bbox.x1 - line.bbox.x1).abs() > 0.5
        {
            continue;
        }
        let step = line.bbox.y1 - above.bbox.y1;
        if step <= 0.0 || step > 2.5 * above.bbox.height() {
            continue;
        }
        let Some(width) = body_width(geometry, line.page) else { continue };
        if above.available < WIDE_MEASURE_SHARE * width
            || above.natural < WRAPPED_SHARE * above.available
        {
            continue;
        }
        let Some(ink) = ink(geometry, line) else { continue };
        if ink.size >= DISPLAY_TEXT_PT {
            continue;
        }
        let left = ink.x0 - line.bbox.x0;
        let right = line.bbox.x1 - ink.x1;
        let side = CENTERED_SIDE_SHARE * line.available;
        if left <= side
            || right <= side
            || (left - right).abs() >= (0.03 * line.available).max(2.0)
        {
            continue;
        }
        if !seen.insert((line.page, line.container)) {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            CENTERED_PARAGRAPH,
            Severity::Warning,
            format!(
                "a paragraph on page {} wraps across a {}pt measure and is centred, so \
                 its lines start at different places",
                line.page + 1,
                round1(line.available)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, line.span);
        diagnostic.elem = line.elem.map(|e| geometry.elements[e].name.into());
        diagnostic.page = Some(line.page + 1);
        diagnostic.bbox = Some(above.bbox.union(&line.bbox).rounded());
        diagnostic.hints.push(
            "align running text left or justify it (set align(left) or par(justify: \
             true) inside the block); centre only single lines such as titles and short \
             captions"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn within(geometry: &Geometry, mut cursor: Option<usize>, target: usize) -> bool {
    while let Some(index) = cursor {
        if index == target {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

fn within_named(geometry: &Geometry, mut cursor: Option<usize>, name: &str) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].name == name {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// The distance between this line's baseline and the nearest other line of
/// the same text block.
fn pitch(geometry: &Geometry, line: &Line) -> Option<f64> {
    geometry
        .lines
        .iter()
        .filter(|l| {
            l.page == line.page
                && l.container == line.container
                && (l.bbox.x0 - line.bbox.x0).abs() < 0.5
        })
        .map(|l| (l.bbox.y1 - line.bbox.y1).abs())
        .filter(|d| *d > 1.0)
        .min_by(f64::total_cmp)
}

fn figure_spacing(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, figure) in geometry.elements.iter().enumerate() {
        if figure.name != "figure" {
            continue;
        }
        for page_box in &figure.boxes {
            let (page, frame) = (page_box.page, page_box.bbox);
            let mut inner: Vec<&Line> = geometry
                .lines
                .iter()
                .filter(|l| l.page == page && within(geometry, l.elem, index))
                .collect();
            inner.sort_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1));
            let outer: Vec<&Line> = geometry
                .lines
                .iter()
                .filter(|l| {
                    l.page == page
                        && geometry.is_body(l.region)
                        && !within(geometry, l.elem, index)
                        && !within_named(geometry, l.elem, "heading")
                        && !within_named(geometry, l.elem, "figure")
                        && l.bbox.x0 < frame.x1
                        && frame.x0 < l.bbox.x1
                })
                .collect();

            let before = outer
                .iter()
                .filter(|l| l.bbox.y1 <= frame.y0 + 1.0)
                .max_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1));
            if let Some(line) = before {
                let size = ink(geometry, line).map_or(10.0, |i| i.size);
                let pitch = pitch(geometry, line).unwrap_or(1.3 * size);
                let extra = match inner.first().filter(|f| f.bbox.y0 <= frame.y0 + 0.5 * size) {
                    Some(first) => first.bbox.y1 - line.bbox.y1 - pitch,
                    None => frame.y0 - line.bbox.y1 - 0.25 * size,
                };
                if extra < MIN_FIGURE_EXTRA_EM * size && extra > -size {
                    out.push(spacing_diagnostic(world, geometry, index, page, "above", extra, size));
                }
            }

            let after = outer
                .iter()
                .filter(|l| l.bbox.y0 >= frame.y1 - 1.0)
                .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
            if let Some(line) = after {
                let size = ink(geometry, line).map_or(10.0, |i| i.size);
                let pitch = pitch(geometry, line).unwrap_or(1.3 * size);
                let extra = match inner.last().filter(|l| l.bbox.y1 >= frame.y1 - 0.5 * size) {
                    Some(last) => line.bbox.y1 - last.bbox.y1 - pitch,
                    None => line.bbox.y0 - frame.y1,
                };
                if extra < MIN_FIGURE_EXTRA_EM * size && extra > -size {
                    out.push(spacing_diagnostic(world, geometry, index, page, "below", extra, size));
                }
            }
        }
    }
    out
}

fn spacing_diagnostic(
    world: &LabWorld,
    geometry: &Geometry,
    figure: usize,
    page: usize,
    side: &'static str,
    extra: f64,
    size: f64,
) -> Diagnostic {
    let element = &geometry.elements[figure];
    let mut diagnostic = Diagnostic::new(
        FIGURE_SPACING,
        Severity::Warning,
        format!(
            "a figure on page {} is set off from the text {side} it by only {}pt more \
             than the line spacing, so it reads as part of the paragraph",
            page + 1,
            round1(extra.max(0.0))
        ),
    );
    diagnostic.loc = diag::resolve_span(world, element.span);
    diagnostic.elem = Some("figure".into());
    diagnostic.label = element.label.clone();
    diagnostic.page = Some(page + 1);
    diagnostic.measured = Some(round1(extra));
    diagnostic.limit = Some(round1(MIN_FIGURE_EXTRA_EM * size));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "give figures at least paragraph spacing (show figure: set block(above: 1.2em, \
         below: 1.2em)) instead of reducing it to save space"
            .into(),
    );
    diagnostic
}
