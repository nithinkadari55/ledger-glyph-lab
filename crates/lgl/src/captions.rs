//! A caption that repeats the label and number its figure prints itself.

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, ItemKind};
use crate::world::LabWorld;

pub const DOUBLED_CAPTION_NUMBER: &str = "L094.doubled-caption-number";

/// Captions whose printed text starts with the figure's own label and then a
/// hand-written one of the same kind: "Table 1: Table 1. Mean abundance".
pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for (index, element) in geometry.elements.iter().enumerate() {
        if element.name != "caption" {
            continue;
        }
        let mut text = String::new();
        let mut first = None;
        for item in &geometry.items {
            if item.kind == ItemKind::Text
                && within(geometry, item.elem, index)
                && let Some(part) = &item.text
            {
                text.push_str(part);
                first.get_or_insert(item);
            }
        }
        let Some(item) = first else { continue };
        let Some((printed, kind)) = caption_label(&text, false) else { continue };
        let Some((_, again)) = caption_label(&text[printed..], false) else { continue };
        if again != kind {
            continue;
        }
        let figure = ancestor(geometry, element.parent, "figure")
            .map(|f| geometry.elements[f].span)
            .filter(|span| !span.is_detached());
        let span = figure.unwrap_or_else(|| geometry.span_for(item.span, item.elem));
        let shown: String = text.trim().chars().take(36).collect();
        let mut diagnostic = Diagnostic::new(
            DOUBLED_CAPTION_NUMBER,
            Severity::Warning,
            format!(
                "the caption on page {} reads \"{shown}…\": the figure prints its own label \
                 and number, so the hand-written one repeats it",
                item.page + 1
            ),
        );
        diagnostic.loc = diag::resolve_span(world, span);
        diagnostic.anchor = Some(span);
        diagnostic.page = Some(item.page + 1);
        diagnostic.bbox = element
            .boxes
            .iter()
            .find(|b| b.page == item.page)
            .map(|b| b.bbox.rounded());
        diagnostic.hints.push(
            "remove the hand-written label (such as *Table 1.*) from the caption text; \
             Typst numbers figures and tables itself"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

/// Whether the element is the one at `index` or inside it.
fn within(geometry: &Geometry, mut elem: Option<usize>, index: usize) -> bool {
    while let Some(current) = elem {
        if current == index {
            return true;
        }
        elem = geometry.elements[current].parent;
    }
    false
}

fn ancestor(geometry: &Geometry, mut elem: Option<usize>, name: &str) -> Option<usize> {
    while let Some(current) = elem {
        if geometry.elements[current].name == name {
            return Some(current);
        }
        elem = geometry.elements[current].parent;
    }
    None
}

/// A caption label at the start of `text` ("Table 1.", "Figure 2:", "Fig. 3 —"):
/// its length in bytes including the space after it, and its kind in lower
/// case ("table", "figure"). In source text, `~` is a space too.
pub(crate) fn caption_label(text: &str, source: bool) -> Option<(usize, String)> {
    let space =
        |c: char| matches!(c, ' ' | '\t' | '\u{a0}' | '\u{202f}') || (source && c == '~');
    let mut chars = text.char_indices().peekable();
    while chars.next_if(|&(_, c)| space(c)).is_some() {}
    let mut word = String::new();
    while let Some((_, c)) = chars.next_if(|&(_, c)| c.is_alphabetic()) {
        word.push(c);
    }
    if word.chars().count() < 3 {
        return None;
    }
    chars.next_if(|&(_, c)| c == '.');
    let mut spaced = false;
    while chars.next_if(|&(_, c)| space(c)).is_some() {
        spaced = true;
    }
    if !spaced {
        return None;
    }
    let mut digits = 0;
    loop {
        while chars.next_if(|&(_, c)| c.is_ascii_digit()).is_some() {
            digits += 1;
        }
        // A part after a dot or a dash: 1.2, 3-1.
        let mut ahead = chars.clone();
        if ahead.next_if(|&(_, c)| matches!(c, '.' | '-' | '–')).is_some()
            && ahead.peek().is_some_and(|&(_, c)| c.is_ascii_digit())
        {
            chars = ahead;
            continue;
        }
        break;
    }
    if digits == 0 {
        return None;
    }
    while chars.next_if(|&(_, c)| space(c)).is_some() {}
    chars.next_if(|&(_, c)| matches!(c, ':' | '.' | '—' | '–' | '-' | '|'))?;
    while chars.next_if(|&(_, c)| space(c)).is_some() {}
    let end = chars.peek().map_or(text.len(), |&(i, _)| i);
    let kind = match word.to_lowercase().as_str() {
        "fig" => "figure".to_string(),
        "tab" => "table".to_string(),
        other => other.to_string(),
    };
    Some((end, kind))
}

#[cfg(test)]
mod tests {
    use super::caption_label;

    #[test]
    fn labels_are_read() {
        assert_eq!(caption_label("Table\u{a0}1: Table 1. Mean", false), Some((10, "table".into())));
        assert_eq!(caption_label("Table 1. Mean", false), Some((9, "table".into())));
        assert_eq!(caption_label("Fig. 2 — Flow", false), Some((11, "figure".into())));
        assert_eq!(caption_label("Figure~3.2: x", true), Some((12, "figure".into())));
        assert_eq!(caption_label("Table 1.", true), Some((8, "table".into())));
    }

    #[test]
    fn text_is_no_label() {
        assert_eq!(caption_label("Mean abundance by site", false), None);
        assert_eq!(caption_label("Table 1 shows the means", false), None);
        assert_eq!(caption_label("In 2024: growth", false), None);
        assert_eq!(caption_label("Figure~3", false), None);
    }
}
