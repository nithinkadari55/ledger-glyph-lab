use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, Line, round1};
use crate::world::LabWorld;

pub const BROKEN_ADDRESS: &str = "L049.broken-address";
pub const LINE_THROUGH_TEXT: &str = "L050.line-through-text";

/// The next word must fit with this much to spare before a line end counts
/// as a manual break; word widths are estimated, so near misses are wraps.
const MANUAL_BREAK_MARGIN: f64 = 1.3;

/// A shape thinner than this in one direction is a line: a rule, a tick, a
/// gridline.
const MAX_LINE_THICKNESS_PT: f64 = 1.5;

/// How far into a text run a line must reach before it reads as crossing it.
const MIN_CROSSING_SHARE: f64 = 0.2;

/// A horizontal line closer than this to the ink of text (its descenders or
/// its tops) touches it when printed.
const MIN_LINE_CLEARANCE_EM: f64 = 0.08;

#[derive(Clone, Copy, PartialEq)]
enum Contact {
    Through,
    Touching,
}

pub const MARKUP_LEAK: &str = "L053.markup-in-text";

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = broken_addresses(world, geometry);
    out.extend(lines_through_text(world, geometry));
    out.extend(markup_leaks(world, geometry));
    out
}

/// The first HTML entity or Markdown emphasis marker in text, such as
/// "&nbsp;" or "**bold**", which Typst prints literally.
fn leaked_markup(text: &str) -> Option<&str> {
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        let tail = &rest[start + 1..];
        let end = tail.find(';')?;
        let name = &tail[..end];
        let named = (2..=8).contains(&name.len())
            && name.chars().all(|c| c.is_ascii_alphabetic());
        let numeric = name.len() > 1
            && name.starts_with('#')
            && name[1..].chars().all(|c| c.is_ascii_alphanumeric());
        if named || numeric {
            return Some(&rest[start..start + end + 2]);
        }
        rest = tail;
    }
    text.find("**").map(|i| &text[i..i + 2])
}

/// HTML entities and Markdown markers that ended up in the printed text.
fn markup_leaks(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut reported = std::collections::BTreeSet::new();
    for item in &geometry.items {
        if item.kind != ItemKind::Text || in_code(geometry, item.elem) {
            continue;
        }
        let Some(found) = item.text.as_deref().and_then(leaked_markup) else { continue };
        if !reported.insert((item.page, found.to_string())) {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            MARKUP_LEAK,
            Severity::Error,
            format!(
                "\"{found}\" is printed literally on page {}: Typst does not interpret HTML \
                 entities or Markdown markers",
                item.page + 1
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(item.span, item.elem));
        diagnostic.page = Some(item.page + 1);
        diagnostic.bbox = item.visible().map(|b| b.rounded());
        diagnostic.hints.push(
            "use Typst syntax: ~ for a non-breaking space, the character itself (– — ’ ×) \
             or sym.*, and *bold* or _italic_ markup"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Whether the content is code, where such text is meant literally.
fn in_code(geometry: &Geometry, mut cursor: Option<usize>) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].name == "raw" {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// The text items of a line, from left to right.
fn line_items<'a>(geometry: &'a Geometry, line: &Line) -> Vec<&'a crate::geom::Item> {
    let b = &line.bbox;
    let mut items: Vec<_> = geometry
        .items
        .iter()
        .filter(|i| i.page == line.page && i.kind == ItemKind::Text)
        .filter(|i| {
            let cx = (i.bbox.x0 + i.bbox.x1) / 2.0;
            let cy = (i.bbox.y0 + i.bbox.y1) / 2.0;
            cy >= b.y0 && cy <= b.y1 && cx >= b.x0 - 1.0 && cx <= b.x1 + 1.0
        })
        .collect();
    items.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
    items
}

fn line_text(geometry: &Geometry, line: &Line) -> String {
    line_items(geometry, line)
        .iter()
        .filter_map(|i| i.text.as_deref())
        .collect()
}

/// The approximate width the first word of a line would need at the end of
/// the previous line, measured on the text item that holds it.
fn first_word_width(geometry: &Geometry, line: &Line, word: &str) -> Option<f64> {
    let item = line_items(geometry, line)
        .into_iter()
        .find(|i| i.text.as_deref().is_some_and(|t| !t.trim().is_empty()))?;
    let chars = item.text.as_deref()?.trim_start().chars().count().max(1);
    let share = (word.chars().count() as f64 / chars as f64).min(1.0);
    // Including the word space before it.
    let space = item.font_size.unwrap_or(10.0) / 3.0;
    Some(item.bbox.width() * share + space)
}

/// Whether a run of text without spaces is a web or email address.
fn is_address(token: &str) -> bool {
    token.contains('@') || token.contains("www.") || token.contains("://")
}

/// Whether the end of a line is the start of an international phone number
/// that the next line continues, as in "+49 40 5550" followed by "1200".
fn splits_phone(text: &str, rest: &str) -> bool {
    let numeric = |w: &str| {
        w.chars().any(|c| c.is_ascii_digit())
            && w.chars().all(|c| c.is_ascii_digit() || "+()-./".contains(c))
    };
    let words: Vec<&str> = text.split_whitespace().collect();
    let Some(start) = words.iter().rposition(|w| w.starts_with('+')) else {
        return false;
    };
    words.len() - start <= 4 && words[start..].iter().all(|w| numeric(w)) && numeric(rest)
}

/// Web and email addresses that wrap onto the next line without a hyphen,
/// such as "www.example." followed by "com", and phone numbers split across
/// lines.
fn broken_addresses(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for pair in geometry.lines.windows(2) {
        let (line, next) = (&pair[0], &pair[1]);
        // Consecutive lines of one text block; their innermost elements may
        // differ, as when the second line starts inside a link.
        let step = next.bbox.y1 - line.bbox.y1;
        if line.page != next.page
            || line.container != next.container
            || (line.bbox.x0 - next.bbox.x0).abs() > 1.0
            || line.hyphenated
            || step <= 0.0
            || step > 2.5 * line.bbox.height()
        {
            continue;
        }
        let text = line_text(geometry, line);
        let following = line_text(geometry, next);
        let Some(token) = text.split_whitespace().last() else { continue };
        let Some(rest) = following.split_whitespace().next() else { continue };
        let ends = token
            .chars()
            .last()
            .is_some_and(|c| c.is_alphanumeric() || ".@/_-".contains(c));
        let continues = rest.chars().next().is_some_and(char::is_alphanumeric);
        // A manual line break after an address is no split: the next word
        // would have fit on the line.
        let forced = first_word_width(geometry, next, rest).is_some_and(|width| {
            line.natural + MANUAL_BREAK_MARGIN * width < line.available
        });
        // The next line may simply start another address.
        let fresh = rest.starts_with("www.")
            || rest.contains("://")
            || (rest.contains('@') && !token.contains('@'));
        let address = !text.ends_with(char::is_whitespace)
            && !fresh
            && ends
            && continues
            && is_address(token);
        let phone = splits_phone(&text, rest);
        if forced || !(address || phone) {
            continue;
        }
        let what = if address { "address" } else { "phone number" };
        let shown = if address {
            format!("{token}{rest}")
        } else {
            let words: Vec<&str> = text.split_whitespace().collect();
            let start = words.iter().rposition(|w| w.starts_with('+')).unwrap_or(0);
            format!("{} {rest}", words[start..].join(" "))
        };
        let mut diagnostic = Diagnostic::new(
            BROKEN_ADDRESS,
            Severity::Warning,
            format!(
                "the {what} \"{shown}\" on page {} is split across two lines",
                line.page + 1
            ),
        );
        diagnostic.loc =
            diag::resolve_span(world, geometry.span_for(line.span, line.elem));
        diagnostic.page = Some(line.page + 1);
        diagnostic.bbox = Some(line.bbox.union(&next.bbox).rounded());
        diagnostic.hints.push(
            "keep the address or number on one line: shorten the surrounding text, give \
             it its own line, or wrap it in box() so it cannot break"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn decorated(geometry: &Geometry, mut cursor: Option<usize>) -> bool {
    while let Some(index) = cursor {
        let element = &geometry.elements[index];
        if matches!(element.name, "underline" | "strike" | "overline" | "highlight")
            || element.label.as_deref() == Some(crate::overlap::ALLOW_OVERLAP)
        {
            return true;
        }
        cursor = element.parent;
    }
    false
}

fn in_equation(geometry: &Geometry, mut cursor: Option<usize>) -> bool {
    while let Some(index) = cursor {
        if geometry.elements[index].name == "equation" {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

/// Thin lines (rules, ticks, gridlines) that run through text.
fn lines_through_text(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let mut reported = std::collections::BTreeSet::new();
    for (index, shape) in geometry.items.iter().enumerate() {
        // Fraction bars and radicals are placed by the math font.
        if shape.kind != ItemKind::Shape
            || decorated(geometry, shape.elem)
            || in_equation(geometry, shape.elem)
        {
            continue;
        }
        let Some(line) = shape.visible() else { continue };
        let vertical = line.width() < MAX_LINE_THICKNESS_PT;
        if !vertical && line.height() >= MAX_LINE_THICKNESS_PT {
            continue;
        }
        for (text_index, text) in geometry.items.iter().enumerate() {
            if text.kind != ItemKind::Text
                || text.page != shape.page
                || text.text.as_deref().is_none_or(|t| t.trim().is_empty())
                || decorated(geometry, text.elem)
            {
                continue;
            }
            let Some(bbox) = text.visible() else { continue };
            let size = text.font_size.unwrap_or(10.0);
            let Some(contact) = contact(&line, &bbox, vertical, size) else { continue };
            if !reported.insert(text_index) {
                continue;
            }
            let mut diagnostic = Diagnostic::new(
                LINE_THROUGH_TEXT,
                Severity::Warning,
                format!(
                    "a {} line {} the text \"{}\" on page {}",
                    if vertical { "vertical" } else { "horizontal" },
                    if contact == Contact::Through { "runs through" } else { "touches" },
                    excerpt(text.text.as_deref()),
                    text.page + 1
                ),
            );
            diagnostic.loc = diag::resolve_span(
                world,
                geometry.span_for(geometry.items[index].span, shape.elem),
            );
            diagnostic.page = Some(text.page + 1);
            diagnostic.bbox = Some(bbox.union(&line).rounded());
            diagnostic.measured = Some(round1(if vertical {
                line.height().min(bbox.height())
            } else {
                line.width().min(bbox.width())
            }));
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "move the line or the text so they do not cross: end gridlines and ticks \
                 before labels, or give labels a background"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}

/// How a thin line meets the ink box of a text run: crossing into it, or
/// running along its top or bottom edge without clearance.
fn contact(line: &BBox, text: &BBox, vertical: bool, size: f64) -> Option<Contact> {
    if vertical {
        let shared = line.intersect(text)?;
        let through = line.x0 > text.x0 + 0.5
            && line.x1 < text.x1 - 0.5
            && shared.height() > MIN_CROSSING_SHARE * text.height();
        return through.then_some(Contact::Through);
    }
    let overlap = line.x1.min(text.x1) - line.x0.max(text.x0);
    if overlap <= MIN_CROSSING_SHARE * text.width() {
        return None;
    }
    if line.y0 > text.y0 + MIN_CROSSING_SHARE * text.height()
        && line.y1 < text.y1 - MIN_CROSSING_SHARE * text.height()
    {
        return Some(Contact::Through);
    }
    let clearance = MIN_LINE_CLEARANCE_EM * size;
    let below = line.y0 - text.y1;
    let above = text.y0 - line.y1;
    ((-0.5..clearance).contains(&below) || (-0.5..clearance).contains(&above))
        .then_some(Contact::Touching)
}

fn excerpt(text: Option<&str>) -> String {
    let text = text.unwrap_or_default().trim();
    let mut chars = text.chars();
    let head: String = chars.by_ref().take(24).collect();
    if chars.next().is_some() { format!("{head}…") } else { head }
}
