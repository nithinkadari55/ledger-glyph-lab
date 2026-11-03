//! Things that must stay with what they belong to: a subtitle with its
//! title, an underline with its text, and an arrow with the boxes it links.

use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const DETACHED_SUBTITLE: &str = "L064.detached-subtitle";
pub const DETACHED_UNDERLINE: &str = "L065.detached-underline";
pub const FLOATING_ARROW: &str = "L066.floating-arrow";

/// Text at least this large, and this many times the body size, is display
/// text: titles, headlines, names.
const DISPLAY_PT: f64 = 14.0;
const DISPLAY_RATIO: f64 = 1.4;

/// The document title is looked for in this top share of the first page.
const TITLE_ZONE: f64 = 0.45;

/// A subtitle further below its title than this many title sizes stands
/// apart from it: another title line would fit into the gap. Convention,
/// tunable.
const MAX_SUBTITLE_GAP_EM: f64 = 1.0;

/// An underline further below its text than this many text sizes no longer
/// reads as underlining it. Convention, tunable.
const MAX_UNDERLINE_GAP_EM: f64 = 0.5;

/// An arrow between two boxes must span at least this share of the gap
/// between them to connect them visibly.
const MIN_ARROW_SPAN: f64 = 0.35;

const ARROWS: &str = "→←⟶⟵➜➔➝➞⇒⇐⇨⇦↑↓⬆⬇";

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = detached_subtitle(world, geometry);
    out.extend(detached_underline(world, geometry));
    out.extend(floating_arrows(world, geometry));
    out
}

/// The most common text size, weighted by characters.
fn body_size(geometry: &Geometry) -> Option<f64> {
    let mut weights: BTreeMap<i64, usize> = BTreeMap::new();
    for item in &geometry.items {
        if let (ItemKind::Text, Some(size), Some(text)) =
            (item.kind, item.font_size, &item.text)
        {
            *weights.entry((size * 10.0).round() as i64).or_default() +=
                text.chars().count();
        }
    }
    let (size, _) = weights.into_iter().max_by_key(|&(size, count)| (count, -size))?;
    Some(size as f64 / 10.0)
}

fn is_text(item: &Item) -> bool {
    item.kind == ItemKind::Text
        && item.font_size.is_some()
        && item.text.as_deref().is_some_and(|t| !t.trim().is_empty())
}

fn overlaps(a: &BBox, b: &BBox) -> bool {
    a.x0 < b.x1 && b.x0 < a.x1
}

/// Whether the element or one of its ancestors has one of the `names`.
fn inside(geometry: &Geometry, mut elem: Option<usize>, names: &[&str]) -> bool {
    while let Some(index) = elem {
        let element = &geometry.elements[index];
        if names.contains(&element.name) {
            return true;
        }
        elem = element.parent;
    }
    false
}

/// A visual line of text: items of one size that share a baseline.
struct TextLine {
    bbox: BBox,
    size: f64,
    items: Vec<usize>,
}

fn text_lines(geometry: &Geometry, page: usize) -> Vec<TextLine> {
    let mut lines: Vec<TextLine> = Vec::new();
    for (index, item) in geometry.items.iter().enumerate() {
        if item.page != page || !is_text(item) || !geometry.is_body(item.region) {
            continue;
        }
        let size = item.font_size.unwrap_or_default();
        match lines.iter_mut().find(|line| {
            (line.bbox.y1 - item.bbox.y1).abs() < 0.3 * size
                && (line.size - size).abs() < 0.5
        }) {
            Some(line) => {
                line.bbox = line.bbox.union(&item.bbox);
                line.items.push(index);
            }
            None => lines.push(TextLine { bbox: item.bbox, size, items: vec![index] }),
        }
    }
    lines
}

fn letters(geometry: &Geometry, line: &TextLine) -> usize {
    line.items
        .iter()
        .filter_map(|&i| geometry.items[i].text.as_deref())
        .map(|text| text.chars().filter(|c| c.is_alphabetic()).count())
        .sum()
}

fn report_span(geometry: &Geometry, item: &Item) -> typst::syntax::Span {
    geometry.span_for(item.span, item.elem)
}

/// A subtitle, standfirst or tagline set so far below the document title
/// that it drifts away from it.
fn detached_subtitle(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let (Some(body), Some(page)) = (body_size(geometry), geometry.pages.first()) else {
        return Vec::new();
    };
    let lines = text_lines(geometry, 0);
    let top: Vec<&TextLine> = lines
        .iter()
        .filter(|l| l.bbox.y0 < TITLE_ZONE * page.height && letters(geometry, l) >= 3)
        .collect();
    let Some(big) = top.iter().map(|l| l.size).reduce(f64::max) else {
        return Vec::new();
    };
    if big < DISPLAY_PT.max(DISPLAY_RATIO * body) {
        return Vec::new();
    }

    // The title: the first line of the largest size and the lines that
    // continue it.
    let mut title: Vec<&TextLine> =
        top.iter().copied().filter(|l| (l.size - big).abs() < 0.5).collect();
    title.sort_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
    let mut block = title[0].bbox;
    for line in &title[1..] {
        if line.bbox.y0 - block.y1 < big {
            block = block.union(&line.bbox);
        }
    }

    let Some(next) = lines
        .iter()
        .filter(|l| l.bbox.y0 >= block.y1 - 0.5 && overlaps(&l.bbox, &block))
        .filter(|l| l.size < big - 0.5)
        .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0))
    else {
        return Vec::new();
    };
    let gap = next.bbox.y0 - block.y1;
    let first = &geometry.items[next.items[0]];

    // Only subtitle-like text: larger than the body, not a heading or table.
    // A box or rule in between ends the title block.
    let subtitle = next.size >= body + 0.4
        && !next.items.iter().any(|&i| {
            inside(geometry, geometry.items[i].elem, &["heading", "table", "grid"])
        });
    let between = |bbox: &BBox| {
        block.y1 - 0.5 <= bbox.y0 && bbox.y0 <= next.bbox.y0 && overlaps(bbox, &block)
    };
    let divided = geometry
        .containers
        .iter()
        .any(|c| c.visible && c.page == 0 && between(&c.bbox))
        || geometry
            .items
            .iter()
            .any(|i| i.page == 0 && i.kind == ItemKind::Shape && between(&i.bbox));
    if !subtitle || divided || gap <= MAX_SUBTITLE_GAP_EM * big {
        return Vec::new();
    }

    let mut diagnostic = Diagnostic::new(
        DETACHED_SUBTITLE,
        Severity::Warning,
        format!(
            "the text below the {}pt title on page 1 starts {}pt under it, more than a \
             title line away, so it drifts from the title instead of belonging to it",
            round1(big),
            round1(gap)
        ),
    );
    diagnostic.loc = diag::resolve_span(world, report_span(geometry, first));
    diagnostic.page = Some(1);
    diagnostic.bbox = Some(block.union(&next.bbox).rounded());
    diagnostic.measured = Some(round1(gap));
    diagnostic.limit = Some(round1(MAX_SUBTITLE_GAP_EM * big));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "keep the subtitle close to the title (about 0.3 to 0.6 times the title size \
         between them) and put the larger space after the whole title block"
            .into(),
    );
    vec![diagnostic]
}

/// An underline below display text (a name on a certificate, a headline)
/// that sits far from its text and closer to the text below it.
fn detached_underline(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let Some(body) = body_size(geometry) else { return Vec::new() };
    let display = DISPLAY_PT.max(DISPLAY_RATIO * body);
    let mut out = Vec::new();
    for rule in &geometry.items {
        let bbox = rule.bbox;
        if rule.kind != ItemKind::Shape
            || bbox.height() >= 3.0
            || bbox.width() <= 30.0
            || !geometry.is_body(rule.region)
        {
            continue;
        }
        let near = |item: &&Item| {
            item.page == rule.page && is_text(item) && overlaps(&item.bbox, &bbox)
        };
        let Some(text) = geometry
            .items
            .iter()
            .filter(near)
            .filter(|i| i.bbox.y1 <= bbox.y0 + 0.5)
            .max_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1))
        else {
            continue;
        };
        let Some(after) = geometry
            .items
            .iter()
            .filter(near)
            .filter(|i| i.bbox.y0 >= bbox.y1 - 0.5)
            .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0))
        else {
            continue;
        };
        let size = text.font_size.unwrap_or_default();
        if size < display {
            continue;
        }
        // The whole line of text above the rule.
        let line = geometry
            .items
            .iter()
            .filter(|i| {
                i.page == text.page
                    && is_text(i)
                    && (i.bbox.y1 - text.bbox.y1).abs() < 1.0
                    && (i.font_size.unwrap_or_default() - size).abs() < 0.5
            })
            .fold(text.bbox, |acc, i| acc.union(&i.bbox));
        let width = line.width();
        let centre = (line.x0 + line.x1) / 2.0;
        let underline = (0.6 * width..=1.05 * width).contains(&bbox.width())
            && ((bbox.x0 + bbox.x1) / 2.0 - centre).abs() <= 0.15 * width;
        let above = bbox.y0 - line.y1;
        let below = after.bbox.y0 - bbox.y1;
        if !underline || above <= MAX_UNDERLINE_GAP_EM * size || above <= below {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            DETACHED_UNDERLINE,
            Severity::Warning,
            format!(
                "the line under the {}pt text on page {} sits {}pt below it but only {}pt \
                 above the next text, so it no longer underlines its text",
                round1(size),
                rule.page + 1,
                round1(above),
                round1(below)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, report_span(geometry, rule));
        diagnostic.page = Some(rule.page + 1);
        diagnostic.bbox = Some(line.union(&bbox).rounded());
        diagnostic.measured = Some(round1(above));
        diagnostic.limit = Some(round1(MAX_UNDERLINE_GAP_EM * size));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "move the line up to the text (at most half the text size below it), or \
             leave the same space above and below it if it is a separator"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// A small arrow glyph lost in a wide gap between the boxes it connects.
fn floating_arrows(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let boxes: Vec<&BBox> = geometry
        .containers
        .iter()
        .filter(|c| c.visible)
        .map(|c| &c.bbox)
        .collect();
    let pages: Vec<usize> = geometry
        .containers
        .iter()
        .filter(|c| c.visible)
        .map(|c| c.page)
        .collect();
    let mut out = Vec::new();
    for arrow in &geometry.items {
        let Some(text) = arrow.text.as_deref().map(str::trim) else { continue };
        if arrow.kind != ItemKind::Text
            || text.is_empty()
            || !text.chars().all(|c| ARROWS.contains(c))
        {
            continue;
        }
        let b = arrow.bbox;
        let on_page = |k: &usize| pages[*k] == arrow.page;
        let (mid_x, mid_y) = ((b.x0 + b.x1) / 2.0, (b.y0 + b.y1) / 2.0);
        // Boxes left and right of the arrow, or above and below it.
        let row: Vec<usize> = (0..boxes.len())
            .filter(on_page)
            .filter(|&k| boxes[k].y0 <= mid_y && mid_y <= boxes[k].y1)
            .collect();
        let column: Vec<usize> = (0..boxes.len())
            .filter(on_page)
            .filter(|&k| boxes[k].x0 <= mid_x && mid_x <= boxes[k].x1)
            .collect();
        let horizontal = {
            let left = row
                .iter()
                .filter(|&&k| boxes[k].x1 <= b.x0 + 0.5)
                .map(|&k| boxes[k].x1);
            let right = row
                .iter()
                .filter(|&&k| boxes[k].x0 >= b.x1 - 0.5)
                .map(|&k| boxes[k].x0);
            match (left.reduce(f64::max), right.reduce(f64::min)) {
                (Some(l), Some(r)) => Some((r - l, b.width())),
                _ => None,
            }
        };
        let vertical = {
            let up = column
                .iter()
                .filter(|&&k| boxes[k].y1 <= b.y0 + 0.5)
                .map(|&k| boxes[k].y1);
            let down = column
                .iter()
                .filter(|&&k| boxes[k].y0 >= b.y1 - 0.5)
                .map(|&k| boxes[k].y0);
            match (up.reduce(f64::max), down.reduce(f64::min)) {
                (Some(u), Some(d)) => Some((d - u, b.height())),
                _ => None,
            }
        };
        let Some((gap, length)) = horizontal.or(vertical) else { continue };
        if gap <= 0.0 || length >= MIN_ARROW_SPAN * gap {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            FLOATING_ARROW,
            Severity::Warning,
            format!(
                "the arrow {text:?} on page {} is {}pt long in a {}pt gap between two \
                 boxes, so it floats instead of connecting them",
                arrow.page + 1,
                round1(length),
                round1(gap)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, report_span(geometry, arrow));
        diagnostic.page = Some(arrow.page + 1);
        diagnostic.bbox = Some(b.rounded());
        diagnostic.measured = Some(round1(length / gap));
        diagnostic.limit = Some(MIN_ARROW_SPAN);
        diagnostic.unit = Some("share of gap");
        diagnostic.hints.push(
            "move the boxes closer together, or draw the arrow across the gap (for \
             example a line with an arrow mark that spans the space between the boxes)"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}
