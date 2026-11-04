//! Things that must stay with what they belong to: a subtitle with its
//! title, an underline with its text, and an arrow with the boxes it links.

use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const DETACHED_SUBTITLE: &str = "L064.detached-subtitle";
pub const DETACHED_UNDERLINE: &str = "L065.detached-underline";
pub const FLOATING_ARROW: &str = "L066.floating-arrow";
pub const DROP_CAP_HOLE: &str = "L068.drop-cap-hole";
pub const PAIR_SPACING: &str = "L070.pair-spacing";
pub const DETACHED_LABEL: &str = "L071.detached-label";
pub const TEXT_ON_RULE: &str = "L072.text-on-rule";

/// Text at least this large, and this many times the body size, is display
/// text: titles, headlines, names.
const DISPLAY_PT: f64 = 14.0;
const DISPLAY_RATIO: f64 = 1.4;

/// The document title is looked for in this top share of the first page.
const TITLE_ZONE: f64 = 0.45;

/// A subtitle further below its title than this many title sizes stands
/// apart from it. Measured on 115 generated documents: subtitles up to 0.77
/// title sizes below their title read as part of it, from 0.94 they drift.
const MAX_SUBTITLE_GAP_EM: f64 = 0.9;

/// A line with more characters than this is running text; the title area of
/// the first page ends where running text starts.
const RUNNING_CHARS: usize = 60;

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
    out.extend(drop_cap_holes(world, geometry));
    out.extend(pair_spacing(world, geometry));
    out.extend(detached_label(world, geometry));
    out.extend(text_on_rule(world, geometry));
    out
}

/// The most common text size of running text, weighted by characters. Code
/// and headings do not count; table text does, since some documents are
/// mostly tables.
fn body_size(geometry: &Geometry) -> Option<f64> {
    let mut weights: BTreeMap<i64, usize> = BTreeMap::new();
    for item in &geometry.items {
        if inside(geometry, item.elem, &["raw", "heading"]) {
            continue;
        }
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

fn characters(geometry: &Geometry, line: &TextLine) -> usize {
    line.items
        .iter()
        .filter_map(|&i| geometry.items[i].text.as_deref())
        .map(|text| text.chars().count())
        .sum()
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

/// A subtitle, standfirst or tagline set so far below its title that it
/// drifts away from it. Every display line in the title area of the first
/// page is checked, so a secondary title ("CERTIFICATE" above a larger name)
/// and a subtitle with a standfirst below it are covered too.
fn detached_subtitle(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let (Some(body), Some(page)) = (body_size(geometry), geometry.pages.first()) else {
        return Vec::new();
    };
    let lines = text_lines(geometry, 0);
    let display = DISPLAY_PT.max(DISPLAY_RATIO * body);
    // The title area ends where running text starts.
    let running = lines
        .iter()
        .filter(|l| characters(geometry, l) > RUNNING_CHARS)
        .map(|l| l.bbox.y1)
        .reduce(f64::min)
        .unwrap_or(f64::INFINITY);
    let mut order: Vec<usize> = (0..lines.len()).collect();
    order.sort_by(|&a, &b| lines[a].bbox.y0.total_cmp(&lines[b].bbox.y0));
    let mut used = vec![false; lines.len()];
    let mut out = Vec::new();
    for &k in &order {
        let title = &lines[k];
        if used[k]
            || title.size < display
            || title.bbox.y0 >= TITLE_ZONE * page.height
            || running <= title.bbox.y0
            || letters(geometry, title) < 3
        {
            continue;
        }
        // The title's own lines.
        used[k] = true;
        let mut block = title.bbox;
        for &j in &order {
            let line = &lines[j];
            if !used[j]
                && (line.size - title.size).abs() < 0.5
                && (0.0..title.size).contains(&(line.bbox.y0 - block.y1))
                && overlaps(&line.bbox, &block)
            {
                used[j] = true;
                block = block.union(&line.bbox);
            }
        }
        out.extend(check_title(world, geometry, &lines, block, title.size, body));
    }
    out
}

fn check_title(
    world: &LabWorld,
    geometry: &Geometry,
    lines: &[TextLine],
    block: BBox,
    big: f64,
    body: f64,
) -> Option<Diagnostic> {
    let next = lines
        .iter()
        .filter(|l| l.bbox.y0 >= block.y1 - 0.5 && overlaps(&l.bbox, &block))
        .filter(|l| l.size < big - 0.5)
        .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0))?;
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
        return None;
    }

    // The subtitle's own lines, and the space below them up to the next text,
    // rule or box. A subtitle that is clearly closer to its title than to what
    // follows still reads as part of it.
    let mut sub = next.bbox;
    while let Some(line) = lines.iter().find(|l| {
        (l.size - next.size).abs() < 0.5
            && l.bbox.y0 > sub.y1 - 0.5
            && l.bbox.y0 - sub.y1 < 0.6 * next.size
            && overlaps(&l.bbox, &sub)
    }) {
        sub = sub.union(&line.bbox);
    }
    let below = |bbox: &BBox| bbox.y0 >= sub.y1 - 0.5 && overlaps(bbox, &block);
    let after = lines
        .iter()
        .map(|l| l.bbox)
        .chain(
            geometry
                .containers
                .iter()
                .filter(|c| c.visible && c.page == 0)
                .map(|c| c.bbox),
        )
        .chain(
            geometry
                .items
                .iter()
                .filter(|i| i.page == 0 && i.kind == ItemKind::Shape)
                .map(|i| i.bbox),
        )
        .filter(below)
        .map(|bbox| bbox.y0 - sub.y1)
        .reduce(f64::min);
    if after.is_none_or(|after| gap < after) {
        return None;
    }

    let mut diagnostic = Diagnostic::new(
        DETACHED_SUBTITLE,
        Severity::Warning,
        format!(
            "the text below the {}pt title on page 1 starts {}pt under it, almost a \
             title line away and no closer to the title than to what follows, so it \
             drifts from the title instead of belonging to it",
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
    Some(diagnostic)
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

/// A drop cap reaches at least this many times the body size.
const DROP_CAP_RATIO: f64 = 2.5;

/// Empty space beside a drop cap, in body sizes, from which it reads as a
/// hole: more than one missing line.
const MAX_DROP_CAP_HOLE_EM: f64 = 1.0;

/// The items that are drop caps: a single letter in the body text at least
/// 2.5 times the body size, not a monogram in a box or circle of its own.
pub fn drop_caps(geometry: &Geometry) -> Vec<usize> {
    let Some(body) = body_size(geometry) else { return Vec::new() };
    let mut out = Vec::new();
    for (index, cap) in geometry.items.iter().enumerate() {
        let Some(size) = cap.font_size else { continue };
        let letter = cap.text.as_deref().map(str::trim).unwrap_or_default();
        let mut chars = letter.chars();
        if !is_text(cap)
            || size < DROP_CAP_RATIO * body
            || !chars.next().is_some_and(char::is_alphabetic)
            || chars.next().is_some()
            || !geometry.is_body(cap.region)
        {
            continue;
        }
        let b = cap.bbox;
        let badge = geometry.containers.iter().any(|c| {
            c.visible
                && c.page == cap.page
                && c.bbox.x0 <= b.x0
                && b.x1 <= c.bbox.x1
                && c.bbox.y0 <= b.y0
                && b.y1 <= c.bbox.y1
                && c.bbox.width() * c.bbox.height() < 4.0 * b.width() * b.height()
        });
        if !badge {
            out.push(index);
        }
    }
    out
}

/// The grid or table cells that hold a drop cap (packages like droplet set
/// the letter and its text in neighbouring cells).
pub fn drop_cap_cells(geometry: &Geometry) -> std::collections::BTreeSet<usize> {
    drop_caps(geometry)
        .into_iter()
        .filter_map(|i| {
            crate::structure::enclosing_cell(geometry, geometry.items[i].container)
        })
        .collect()
}

/// A drop cap taller than the lines beside it: its paragraph is too short,
/// so the big letter leaves a hole in the text.
fn drop_cap_holes(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let Some(body) = body_size(geometry) else { return Vec::new() };
    let mut out = Vec::new();
    for index in drop_caps(geometry) {
        let cap = &geometry.items[index];
        let letter = cap.text.as_deref().map(str::trim).unwrap_or_default();
        let b = cap.bbox;
        let beside: Vec<&Item> = geometry
            .items
            .iter()
            .filter(|i| {
                is_text(i)
                    && i.page == cap.page
                    && i.bbox.x0 >= b.x1 - 1.0
                    && i.bbox.x0 < b.x1 + 3.0 * body
                    && i.bbox.y0 < b.y1
                    && i.bbox.y1 > b.y0
            })
            .collect();
        let Some(low) = beside.iter().map(|i| i.bbox.y1).reduce(f64::max) else {
            continue;
        };
        let hole = b.y1 - low;
        if hole <= MAX_DROP_CAP_HOLE_EM * body {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            DROP_CAP_HOLE,
            Severity::Warning,
            format!(
                "the drop cap {letter:?} on page {} is {}pt taller than the text beside it, \
                 leaving a hole: its paragraph is too short for it",
                cap.page + 1,
                round1(hole)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, report_span(geometry, cap));
        diagnostic.page = Some(cap.page + 1);
        diagnostic.bbox = Some(b.rounded());
        diagnostic.measured = Some(round1(hole));
        diagnostic.limit = Some(round1(MAX_DROP_CAP_HOLE_EM * body));
        diagnostic.unit = Some("pt");
        diagnostic.hints.push(
            "start the drop cap on a paragraph long enough to run beside its full height, \
             or make the drop cap span fewer lines"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// At least this many pairs in a row make a repeated pattern.
const MIN_PAIRS: usize = 3;

/// The space inside a pair must exceed the space between pairs by this much,
/// and by this factor, to count as inverted.
const PAIR_SLACK_PT: f64 = 1.5;
const PAIR_RATIO: f64 = 1.25;

/// A run of text in one style: a line or a paragraph.
struct Unit {
    key: (i64, [i32; 4]),
    bbox: BBox,
    size: f64,
    item: usize,
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Repeated pairs (command and description, label and value, term and
/// definition) whose second part sits closer to the next pair than to its own
/// first part, so every pair reads wrongly grouped. Headings and code are left
/// out: heading spacing has its own rules. Measured on 121 generated
/// documents: 13 have runs of pairs, the 2 inverted ones were confirmed by eye.
fn pair_spacing(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for page in 0..geometry.pages.len() {
        // Visual lines: text items sharing a baseline.
        let mut lines: Vec<Unit> = Vec::new();
        for (index, item) in geometry.items.iter().enumerate() {
            if item.page != page
                || !is_text(item)
                || !geometry.is_body(item.region)
                || inside(geometry, item.elem, &["raw", "heading"])
            {
                continue;
            }
            let size = item.font_size.unwrap_or_default();
            let b = item.bbox;
            if let Some(line) = lines.iter_mut().find(|l| {
                (l.bbox.y1 - b.y1).abs() < 0.3 * size
                    && l.bbox.x0 - 2.0 <= b.x0
                    && b.x0 <= l.bbox.x1 + 30.0
            }) {
                line.bbox = line.bbox.union(&b);
                continue;
            }
            let color =
                item.color.unwrap_or_default().map(|c| (c * 100.0).round() as i32);
            lines.push(Unit {
                key: ((size * 10.0).round() as i64, color),
                bbox: b,
                size,
                item: index,
            });
        }
        lines.sort_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));

        // Paragraphs: consecutive lines of one style and left edge.
        let mut paragraphs: Vec<Unit> = Vec::new();
        for line in lines {
            if let Some(p) = paragraphs.iter_mut().rev().find(|p| {
                p.key == line.key
                    && (p.bbox.x0 - line.bbox.x0).abs() < 3.0
                    && (-1.0..0.5 * line.size).contains(&(line.bbox.y0 - p.bbox.y1))
            }) {
                p.bbox = p.bbox.union(&line.bbox);
                continue;
            }
            paragraphs.push(line);
        }

        // Columns: paragraphs sharing a left edge.
        let mut columns: BTreeMap<i64, Vec<&Unit>> = BTreeMap::new();
        for p in &paragraphs {
            columns.entry((p.bbox.x0 / 4.0).round() as i64).or_default().push(p);
        }
        for column in columns.values_mut() {
            column.sort_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0));
            let mut k = 0;
            while k + 2 * MIN_PAIRS - 1 < column.len() {
                let (a, b) = (column[k].key, column[k + 1].key);
                if a == b {
                    k += 1;
                    continue;
                }
                let start = k;
                let style =
                    move |j: usize| if (j - start).is_multiple_of(2) { a } else { b };
                let mut j = k;
                while j + 1 < column.len()
                    && column[j].key == style(j)
                    && column[j + 1].key == style(j + 1)
                {
                    j += 1;
                }
                let run = &column[k..=j];
                // A run of pairs starts with a first part and ends with a second
                // part; an odd run carries an intro or a stray element, which
                // would pair the parts the wrong way round.
                if run.len() < 2 * MIN_PAIRS || !run.len().is_multiple_of(2) {
                    k += 1;
                    continue;
                }
                let gaps: Vec<f64> =
                    run.windows(2).map(|w| w[1].bbox.y0 - w[0].bbox.y1).collect();
                let within = median(gaps.iter().step_by(2).copied().collect());
                let between = median(gaps.iter().skip(1).step_by(2).copied().collect());
                if within > between + PAIR_SLACK_PT && within > PAIR_RATIO * between {
                    let second = &geometry.items[run[1].item];
                    let mut diagnostic = Diagnostic::new(
                        PAIR_SPACING,
                        Severity::Warning,
                        format!(
                            "{} repeated pairs on page {} are {}pt apart inside each pair \
                             but only {}pt from the next pair, so each second part reads \
                             as belonging to the pair below",
                            run.len() / 2,
                            page + 1,
                            round1(within),
                            round1(between)
                        ),
                    );
                    diagnostic.loc =
                        diag::resolve_span(world, report_span(geometry, second));
                    diagnostic.page = Some(page + 1);
                    diagnostic.bbox =
                        Some(run[0].bbox.union(&run[run.len() - 1].bbox).rounded());
                    diagnostic.measured = Some(round1(within));
                    diagnostic.limit = Some(round1(between));
                    diagnostic.unit = Some("pt");
                    diagnostic.hints.push(
                        "keep each label or command closer to its own value or \
                         description than to the next pair: less space inside a pair, \
                         more space between pairs"
                            .into(),
                    );
                    out.push(diagnostic);
                }
                k = j;
            }
        }
    }
    out
}

/// A label above a title (a kicker like "FEATURE · CITIES") further above it
/// than this many times its own size floats free of the title. Measured on
/// 121 generated documents: labels at 3.1 to 4.3 times their size read as
/// part of the title; at 6.1 and 7.6 they float.
const MAX_LABEL_GAP: f64 = 5.0;

/// A small label above the document title, so far above it that it floats.
fn detached_label(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
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
    let Some(title) = top
        .iter()
        .filter(|l| (l.size - big).abs() < 0.5)
        .min_by(|a, b| a.bbox.y0.total_cmp(&b.bbox.y0))
    else {
        return Vec::new();
    };
    let Some(label) = lines
        .iter()
        .filter(|l| l.bbox.y1 <= title.bbox.y0 + 0.5 && overlaps(&l.bbox, &title.bbox))
        .filter(|l| l.size < big - 0.5)
        .max_by(|a, b| a.bbox.y1.total_cmp(&b.bbox.y1))
    else {
        return Vec::new();
    };
    let item = &geometry.items[label.items[0]];
    let gap = title.bbox.y0 - label.bbox.y1;
    let between = |b: &BBox| {
        label.bbox.y1 - 0.5 <= b.y0 && b.y1 <= title.bbox.y0 + 0.5 && overlaps(b, &title.bbox)
    };
    // A rule or box in between makes the label a separate header.
    let divided = geometry
        .containers
        .iter()
        .any(|c| c.visible && c.page == 0 && between(&c.bbox))
        || geometry
            .items
            .iter()
            .any(|i| i.page == 0 && i.kind == ItemKind::Shape && between(&i.bbox));
    if divided
        || inside(geometry, item.elem, &["heading", "table", "grid"])
        || gap <= MAX_LABEL_GAP * label.size
    {
        return Vec::new();
    }
    let mut diagnostic = Diagnostic::new(
        DETACHED_LABEL,
        Severity::Warning,
        format!(
            "the {}pt label above the title on page 1 sits {}pt above it, more than {} \
             times its own size, so it floats free of the title",
            round1(label.size),
            round1(gap),
            MAX_LABEL_GAP
        ),
    );
    diagnostic.loc = diag::resolve_span(world, report_span(geometry, item));
    diagnostic.page = Some(1);
    diagnostic.bbox = Some(label.bbox.union(&title.bbox).rounded());
    diagnostic.measured = Some(round1(gap));
    diagnostic.limit = Some(round1(MAX_LABEL_GAP * label.size));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "set the label about two to four times its own size above the title".into(),
    );
    vec![diagnostic]
}

/// A text line sitting on a horizontal rule that runs on both sides of it,
/// level with its bottom: two blocks have collapsed into each other (often a
/// v(1fr) that shrank to nothing).
fn text_on_rule(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let rules: Vec<&Item> = geometry
        .items
        .iter()
        .filter(|r| r.kind == ItemKind::Shape && r.bbox.height() < 3.0 && r.bbox.width() > 30.0)
        .collect();
    let mut out = Vec::new();
    for text in &geometry.items {
        let Some(size) = text.font_size else { continue };
        let visible = text.text.as_deref().map(str::trim).unwrap_or_default();
        if !is_text(text) || visible.chars().count() < 3 {
            continue;
        }
        let t = text.bbox;
        let level = |r: &&&Item| {
            r.page == text.page && (t.y1 - 0.3 * size..=t.y1 + 1.0).contains(&r.bbox.y0)
        };
        let left = rules
            .iter()
            .filter(level)
            .any(|r| (0.0..=2.0 * size).contains(&(t.x0 - r.bbox.x1)));
        let right = rules
            .iter()
            .filter(level)
            .any(|r| (0.0..=2.0 * size).contains(&(r.bbox.x0 - t.x1)));
        if !left || !right {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            TEXT_ON_RULE,
            Severity::Warning,
            format!(
                "text on page {} sits on a line that runs on both sides of it: the block \
                 below has run into the text",
                text.page + 1
            ),
        );
        diagnostic.loc = diag::resolve_span(world, report_span(geometry, text));
        diagnostic.page = Some(text.page + 1);
        diagnostic.bbox = Some(t.rounded());
        diagnostic.hints.push(
            "leave space between the text and the block with the line (a v(1fr) shrinks to \
             nothing when the page is full; add a fixed gap such as v(1.5em) next to it), or \
             shorten the text"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}
