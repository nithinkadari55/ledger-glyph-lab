use std::collections::{HashMap, HashSet};

use typst::syntax::Span;
use typst_layout::PagedDocument;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Container, Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const PAGE_OVERFLOW: &str = "L001.page-overflow";
pub const CONTAINER_OVERFLOW: &str = "L002.container-overflow";
pub const MARGIN_INTRUSION: &str = "L003.margin-intrusion";
pub const MARGINAL_OVERFLOW: &str = "L018.marginal-overflow";
pub const COLUMN_OVERFLOW: &str = "L032.column-overflow";
pub const LINE_OVERFLOW: &str = "L004.line-overflow";
pub const LOOSE_LINE: &str = "L005.loose-line";

/// Tolerance for geometric bounds, in points.
const TOLERANCE_PT: f64 = 0.5;

/// Text may hang into the margin by design (Typst hangs dashes, hyphens and
/// punctuation at line ends), and glyphs of large text can reach slightly
/// past their line start, so text gets a tolerance relative to its size. A
/// letter or digit at the right end of the text gets a small tolerance only:
/// nothing hangs it there.
const TEXT_TOLERANCE_EM: f64 = 0.35;

/// Tolerance for a letter or digit at the right end of text. Measured on 115
/// generated documents: glyph advances reach up to 0.11em past a line end;
/// a clipped table header reached 0.26em.
const LETTER_TOLERANCE_EM: f64 = 0.15;

/// Extra space between characters of a justified line from which letter
/// spacing becomes visible.
const LOOSE_EXTRA_PT: f64 = 0.5;

pub fn run(
    world: &LabWorld,
    _document: &PagedDocument,
    geometry: &Geometry,
) -> Vec<Diagnostic> {
    let mut out = bounds(world, geometry);
    out.extend(loose_lines(world, geometry));
    out.extend(crate::overlap::run(world, geometry));
    out.extend(crate::whitespace::run(world, geometry));
    out.extend(crate::resources::run(world, geometry));
    out.extend(crate::typography::run(world, geometry));
    out.extend(crate::spacing::run(world, geometry));
    out.extend(crate::structure::run(world, geometry));
    out.extend(crate::pagecheck::run(world, geometry));
    out.extend(crate::quality::run(world, geometry));
    out.extend(crate::grid::run(world, geometry));
    out.extend(crate::rhythm::run(world, geometry));
    out.extend(crate::boxes::run(world, geometry));
    out.extend(crate::composition::run(world, geometry));
    out.extend(crate::contrast::run(world, geometry));
    out.extend(crate::marks::run(world, geometry));
    out.extend(crate::titles::run(world, geometry));
    out.extend(crate::edges::run(world, geometry));
    out
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Anchor {
    Line(usize),
    Container(usize),
    Span(Span),
    Item(usize),
}

struct Finding {
    rule: &'static str,
    page: usize,
    amount: f64,
    side: &'static str,
    span: Span,
    elem: Option<String>,
    label: Option<String>,
    bbox: BBox,
}

struct Subject {
    page: usize,
    bbox: BBox,
    /// The tolerance above and below.
    tolerance: f64,
    /// The tolerance on the left and on the right.
    sides: [f64; 2],
    container: Option<usize>,
    region: Option<usize>,
    column: Option<usize>,
    span: Span,
    elem: Option<String>,
    label: Option<String>,
}

type Findings = HashMap<(&'static str, Anchor), Finding>;

fn record(
    findings: &mut Findings,
    (rule, anchor, (amount, side)): Classified,
    subject: &Subject,
) {
    let entry = findings.entry((rule, anchor)).or_insert(Finding {
        rule,
        page: subject.page,
        amount,
        side,
        span: subject.span,
        elem: subject.elem.clone(),
        label: subject.label.clone(),
        bbox: subject.bbox,
    });
    if amount > entry.amount {
        entry.amount = amount;
        entry.side = side;
        entry.bbox = subject.bbox;
    }
}

fn bounds(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut findings = Findings::new();
    let mut flagged_containers = HashSet::new();

    for (index, container) in geometry.containers.iter().enumerate() {
        if container.kind != "cell" && !container.sized && !container.visible {
            continue;
        }
        let subject = Subject {
            page: container.page,
            bbox: container.bbox,
            tolerance: TOLERANCE_PT,
            sides: [TOLERANCE_PT; 2],
            container: container.parent,
            region: container.region,
            column: container.column,
            span: container.span,
            elem: Some(container.kind.to_string()),
            label: container.elem.and_then(|e| geometry.elements[e].label.clone()),
        };
        if let Some(classified) = classify(geometry, &subject, None) {
            flagged_containers.insert(index);
            record(&mut findings, classified, &subject);
        }
    }

    for (index, item) in geometry.items.iter().enumerate() {
        if item.kind == ItemKind::Link {
            continue;
        }
        let Some(bbox) = item.visible() else { continue };
        let tolerance = match (item.kind, item.font_size) {
            (ItemKind::Text, Some(size)) => TOLERANCE_PT.max(TEXT_TOLERANCE_EM * size),
            _ => TOLERANCE_PT,
        };
        // A trailing space is part of the item's box but invisible, so only
        // text that really ends in a letter or digit is held to the tighter
        // tolerance on the right.
        let last = item.text.as_deref().and_then(|t| t.chars().next_back());
        let sides = match (last, item.font_size) {
            (Some(c), Some(size)) if item.kind == ItemKind::Text && c.is_alphanumeric() => {
                [tolerance, TOLERANCE_PT.max(LETTER_TOLERANCE_EM * size)]
            }
            _ => [tolerance; 2],
        };
        let element = item.elem.map(|e| &geometry.elements[e]);
        // A container's own fill and stroke belong to it, and a stroke is
        // centred on its edge, so they are checked against the parent.
        let mut container = item.container;
        if item.kind == ItemKind::Shape
            && let Some(own) = container
            && geometry.containers[own].span == item.span
        {
            container = geometry.containers[own].parent;
        }
        let subject = Subject {
            page: item.page,
            bbox,
            tolerance,
            sides,
            container,
            region: item.region,
            column: item.column,
            span: item.span,
            elem: element.map(|e| e.name.to_string()),
            label: element.and_then(|e| e.label.clone()),
        };
        let line = (item.kind == ItemKind::Text)
            .then(|| overfull_line(geometry, item.page, &bbox))
            .flatten();
        let Some((rule, anchor, excess)) = classify(geometry, &subject, line) else {
            continue;
        };
        if rule != CONTAINER_OVERFLOW
            && rule != LINE_OVERFLOW
            && inside_flagged(geometry, item.container, &flagged_containers)
        {
            continue;
        }
        let anchor = match anchor {
            Anchor::Span(span) if span.is_detached() => Anchor::Item(index),
            other => other,
        };
        record(&mut findings, (rule, anchor, excess), &subject);
    }

    let mut out: Vec<Diagnostic> = findings
        .into_iter()
        .map(|((_, anchor), finding)| to_diagnostic(world, geometry, anchor, finding))
        .collect();
    out.sort_by(|a, b| {
        a.page
            .cmp(&b.page)
            .then_with(|| a.rule.cmp(&b.rule))
            .then_with(|| a.message.cmp(&b.message))
    });
    out
}

type Classified = (&'static str, Anchor, (f64, &'static str));

fn classify(
    geometry: &Geometry,
    subject: &Subject,
    line: Option<usize>,
) -> Option<Classified> {
    let page = &geometry.pages[subject.page];
    let page_excess = subject.bbox.excess(&page.bounds());
    let bounded = bounded_container(geometry, subject.container);
    let container_excess =
        bounded.map(|c| subject.bbox.excess(&geometry.containers[c].bbox));
    let region_excess =
        subject.region.map(|r| subject.bbox.excess(&geometry.regions[r].bbox));

    let exceeds_container =
        bounded.is_some_and(|c| exceeds(subject, &geometry.containers[c].bbox));
    let exceeds_page = page_excess.0 > TOLERANCE_PT;
    let exceeds_region = subject
        .region
        .is_some_and(|r| exceeds(subject, &geometry.regions[r].bbox));
    let column_excess = subject.column.map(|c| {
        let column = &geometry.columns[c].bbox;
        let (left, right) = (column.x0 - subject.bbox.x0, subject.bbox.x1 - column.x1);
        if left > right { (left, "left") } else { (right, "right") }
    });
    let exceeds_column = subject.column.is_some_and(|c| {
        let column = &geometry.columns[c].bbox;
        column.x0 - subject.bbox.x0 > subject.sides[0]
            || subject.bbox.x1 - column.x1 > subject.sides[1]
    });
    if !exceeds_container && !exceeds_page && !exceeds_region && !exceeds_column {
        return None;
    }

    if let Some(line) = line {
        let excess = (geometry.lines[line].overflow, "right");
        return Some((LINE_OVERFLOW, Anchor::Line(line), excess));
    }
    if exceeds_container {
        let index = bounded?;
        return Some((CONTAINER_OVERFLOW, Anchor::Container(index), container_excess?));
    }
    let marginal = subject.region.is_some() && !geometry.is_body(subject.region);
    if marginal && exceeds_region {
        return Some((MARGINAL_OVERFLOW, Anchor::Span(subject.span), region_excess?));
    }
    if exceeds_page {
        return Some((PAGE_OVERFLOW, Anchor::Span(subject.span), page_excess));
    }
    if exceeds_column {
        return Some((COLUMN_OVERFLOW, Anchor::Span(subject.span), column_excess?));
    }
    let rule = if geometry.is_body(subject.region) {
        MARGIN_INTRUSION
    } else {
        MARGINAL_OVERFLOW
    };
    Some((rule, Anchor::Span(subject.span), region_excess?))
}

/// Whether the subject extends beyond `bounds` by more than its tolerance on
/// any side.
fn exceeds(subject: &Subject, bounds: &BBox) -> bool {
    let b = &subject.bbox;
    bounds.x0 - b.x0 > subject.sides[0]
        || b.x1 - bounds.x1 > subject.sides[1]
        || bounds.y0 - b.y0 > subject.tolerance
        || b.y1 - bounds.y1 > subject.tolerance
}

/// The nearest container whose bounds the author can see or chose: a cell, a
/// container with an explicit size, or one with a fill or stroke. Plain
/// automatic containers may legitimately be exceeded, e.g. by hanging indents
/// and list markers. Returns `None` if a clipping container comes first,
/// since nothing beyond it is visible.
fn bounded_container(geometry: &Geometry, mut cursor: Option<usize>) -> Option<usize> {
    while let Some(index) = cursor {
        let container = &geometry.containers[index];
        if container.clip {
            return None;
        }
        if container.kind == "cell" || container.sized || container.visible {
            return Some(index);
        }
        cursor = container.parent;
    }
    None
}

fn overfull_line(geometry: &Geometry, page: usize, bbox: &BBox) -> Option<usize> {
    let center = (bbox.y0 + bbox.y1) / 2.0;
    geometry.lines.iter().position(|line| {
        line.page == page
            && line.overflow > TOLERANCE_PT
            && line.bbox.y0 <= center
            && center <= line.bbox.y1
            && bbox.x1 > line.bbox.x0
            && bbox.x0 < line.bbox.x1 + line.overflow + TOLERANCE_PT
    })
}

fn inside_flagged(
    geometry: &Geometry,
    mut cursor: Option<usize>,
    flagged: &HashSet<usize>,
) -> bool {
    while let Some(index) = cursor {
        if flagged.contains(&index) {
            return true;
        }
        cursor = geometry.containers[index].parent;
    }
    false
}

fn to_diagnostic(
    world: &LabWorld,
    geometry: &Geometry,
    anchor: Anchor,
    finding: Finding,
) -> Diagnostic {
    let page = finding.page + 1;
    let amount = round1(finding.amount);
    let (severity, message, hint, span, elem) = match (finding.rule, anchor) {
        (LINE_OVERFLOW, Anchor::Line(line)) => {
            let line = &geometry.lines[line];
            (
                Severity::Error,
                format!(
                    "line is {amount}pt wider than the {:.1}pt available on page {page}",
                    line.available
                ),
                "allow a break inside the long word or URL (enable hyphenation, \
                     insert #sym.zws or a soft hyphen), or reduce the content's width",
                line.span,
                finding.elem,
            )
        }
        (CONTAINER_OVERFLOW, Anchor::Container(container)) => {
            let container = &geometry.containers[container];
            let kind = container_name(geometry, container);
            (
                Severity::Error,
                format!(
                    "content extends {amount}pt beyond the {} edge of its {kind} \
                         ({:.1}pt x {:.1}pt) on page {page}",
                    finding.side,
                    container.bbox.width(),
                    container.bbox.height()
                ),
                match finding.side {
                    "left" | "right" => {
                        "give the container more width (auto, fr or %), let the \
                             content wrap, or enable clip if cutting it off is intended"
                    }
                    _ => {
                        "give the container more height (auto), reduce the content, \
                             or enable clip if cutting it off is intended"
                    }
                },
                if finding.span.is_detached() { container.span } else { finding.span },
                Some(kind.to_string()),
            )
        }
        (PAGE_OVERFLOW, _) => (
            Severity::Error,
            format!(
                "content extends {amount}pt beyond the {} edge of page {page}",
                finding.side
            ),
            match finding.side {
                "left" | "right" => {
                    "reduce the width: use relative (%) or fractional (fr) sizes, or \
                         let the content wrap"
                }
                _ => "let the content break across pages, or reduce its height",
            },
            finding.span,
            finding.elem,
        ),
        (COLUMN_OVERFLOW, _) => (
            Severity::Error,
            format!(
                "content extends {amount}pt beyond the {} edge of its column on page \
                 {page}, into the gutter or the next column",
                finding.side
            ),
            "keep the content within the column width: use relative (%) or \
             fractional (fr) sizes, let it wrap, or place it across columns with \
             place(scope: \"parent\", float: true)",
            finding.span,
            finding.elem,
        ),
        (MARGINAL_OVERFLOW, _) => (
            Severity::Error,
            format!(
                "header or footer content extends {amount}pt beyond the {} edge of \
                 its area on page {page}",
                finding.side
            ),
            "shorten the header or footer, reduce its font size, or enlarge the page \
             margin that holds it",
            finding.span,
            finding.elem,
        ),
        _ => (
            Severity::Error,
            format!(
                "content extends {amount}pt into the {} margin of page {page}",
                finding.side
            ),
            "keep the content within the text area: reduce its size or use \
                 relative (%) or fractional (fr) sizes",
            finding.span,
            finding.elem,
        ),
    };

    let mut out = Diagnostic::new(finding.rule, severity, message);
    out.loc = diag::resolve_span(world, span);
    out.anchor = Some(match anchor {
        Anchor::Container(index) => geometry.containers[index].span,
        _ => finding.span,
    });
    out.side = Some(finding.side);
    out.elem = elem;
    out.label = finding.label;
    out.page = Some(page);
    out.bbox = Some(finding.bbox.rounded());
    out.measured = Some(amount);
    out.limit = Some(TOLERANCE_PT);
    out.unit = Some("pt");
    out.hints.push(hint.into());
    out
}

fn loose_lines(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    geometry
        .lines
        .iter()
        .filter(|line| line.justify && line.extra >= LOOSE_EXTRA_PT)
        .map(|line| {
            let page = line.page + 1;
            let extra = (line.extra * 100.0).round() / 100.0;
            let mut out = Diagnostic::new(
                LOOSE_LINE,
                Severity::Warning,
                format!(
                    "justified line on page {page} ran out of stretchable space and \
                     adds {extra}pt between characters"
                ),
            );
            out.loc = diag::resolve_span(world, line.span);
            out.page = Some(page);
            out.bbox = Some(line.bbox.rounded());
            out.measured = Some(extra);
            out.limit = Some(LOOSE_EXTRA_PT);
            out.unit = Some("pt");
            out.hints.push(
                "enable hyphenation (set text(lang: ..., hyphenate: true)), widen the \
                 text column, or set narrow columns ragged-right (par(justify: false)); \
                 rewording the text is the last resort"
                    .into(),
            );
            out
        })
        .collect()
}

/// How to call a container in a message. Table and grid cells wrap their body
/// in a block of the same size, which the author knows as the cell.
fn container_name(geometry: &Geometry, container: &Container) -> &'static str {
    match container.parent.map(|p| &geometry.containers[p]) {
        Some(parent)
            if container.kind == "block"
                && parent.kind == "cell"
                && same_box(&parent.bbox, &container.bbox) =>
        {
            "cell"
        }
        _ => container.kind,
    }
}

fn same_box(a: &BBox, b: &BBox) -> bool {
    (a.x0 - b.x0).abs() <= TOLERANCE_PT
        && (a.y0 - b.y0).abs() <= TOLERANCE_PT
        && (a.x1 - b.x1).abs() <= TOLERANCE_PT
        && (a.y1 - b.y1).abs() <= TOLERANCE_PT
}
