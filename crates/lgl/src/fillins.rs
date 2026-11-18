//! A short label ending in a colon ("Answer:", "Name:") left at the end of a
//! running line while its line to write on starts the next line. Measured in
//! batch 18 on 450 generated documents: a label with its line on the same
//! baseline (a Name and Date row) sits within half a size of the label's box;
//! a split one has its line a full line lower, at the start of the line.

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const SPLIT_FILL_IN: &str = "L096.split-fill-in";

/// A label is short: at most this many characters and words.
const MAX_LABEL_CHARS: usize = 24;
const MAX_LABEL_WORDS: usize = 3;

/// A line to write on is thinner than this and at least this wide, in points.
const RULE_MAX_HEIGHT_PT: f64 = 1.5;
const RULE_MIN_WIDTH_PT: f64 = 30.0;

/// A line whose top lies within this many label sizes under the label's box
/// is on the label's own row (drawn on its baseline).
const SAME_ROW_EM: f64 = 0.5;

/// The line that starts the next line lies at most this many label sizes
/// under the label's box, and starts within this many sizes of the start of
/// the label's line.
const NEXT_LINE_EM: f64 = 2.5;
const LINE_START_EM: f64 = 4.0;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let mut out = Vec::new();
    for (index, label) in items.iter().enumerate() {
        if !is_text(label) {
            continue;
        }
        let shown = label.text.as_deref().unwrap_or_default().trim();
        if !shown.ends_with(':')
            || shown.chars().count() > MAX_LABEL_CHARS
            || shown.split_whitespace().count() > MAX_LABEL_WORDS
        {
            continue;
        }
        let b = label.bbox;
        let size = label.font_size.unwrap_or(10.0);
        let page = label.page;
        let on_row = |o: &Item| o.page == page && o.bbox.y0 < b.y1 && o.bbox.y1 > b.y0;
        let on_baseline = |o: &Item| {
            o.page == page && b.y0 < o.bbox.y0 && o.bbox.y0 <= b.y1 + SAME_ROW_EM * size
        };
        // Nothing follows the label on its row: no text, no line.
        let followed = items.iter().enumerate().any(|(k, o)| {
            k != index
                && o.bbox.x0 >= b.x1 - 0.5
                && ((is_text(o) && on_row(o)) || (is_rule(o) && (on_row(o) || on_baseline(o))))
        });
        if followed {
            continue;
        }
        // Running text before the label on its row.
        let start = items
            .iter()
            .enumerate()
            .filter(|&(k, o)| k != index && is_text(o) && on_row(o) && o.bbox.x1 <= b.x0 + 0.5)
            .map(|(_, o)| o.bbox.x0)
            .fold(f64::INFINITY, f64::min);
        if !start.is_finite() {
            continue;
        }
        // Its line to write on starts the next line, at the left.
        let Some(rule) = items.iter().find(|r| {
            let rb = r.bbox;
            is_rule(r)
                && r.page == page
                && rb.y0 > b.y1 + SAME_ROW_EM * size
                && rb.y0 <= b.y1 + NEXT_LINE_EM * size
                && rb.x0 < b.x0 - 0.4 * (b.x1 - start + 1.0)
                && (rb.x0 - start).abs() <= LINE_START_EM * size
        }) else {
            continue;
        };
        let mut diagnostic = Diagnostic::new(
            SPLIT_FILL_IN,
            Severity::Warning,
            format!(
                "the label \"{shown}\" on page {} ends its line, and the line to write on \
                 starts the next line {}pt further left, so the two are split apart",
                page + 1,
                round1(b.x0 - rule.bbox.x0)
            ),
        );
        diagnostic.loc = diag::resolve_span(world, geometry.span_for(label.span, label.elem));
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = Some(b.union(&rule.bbox).rounded());
        diagnostic.hints.push(
            "keep a label and its line to write on together: put both in one box, as in \
             box[Answer: #box(width: 3cm, stroke: (bottom: 0.5pt))], or give the label and \
             its line a line of their own below the question"
                .into(),
        );
        out.push(diagnostic);
    }
    out
}

fn is_text(item: &Item) -> bool {
    item.kind == ItemKind::Text && item.text.as_deref().is_some_and(|t| !t.trim().is_empty())
}

fn is_rule(item: &Item) -> bool {
    item.kind == ItemKind::Shape
        && item.bbox.height() < RULE_MAX_HEIGHT_PT
        && item.bbox.width() >= RULE_MIN_WIDTH_PT
}
