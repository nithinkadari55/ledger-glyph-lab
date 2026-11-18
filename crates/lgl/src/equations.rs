//! A short inline equation broken across two lines ("36 ÷ 6 =" at the end
//! of one line, "6" at the start of the next). Measured in batch 18 on 450
//! generated documents: every inline equation broken across lines and at
//! most 6 text sizes wide read as torn apart (16 in 13 documents: worked
//! steps of worksheets and short settings such as "α = 1/L" in papers);
//! longer ones break after a relation as in any typeset text.

use std::collections::{BTreeMap, BTreeSet};

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const BROKEN_EQUATION: &str = "L097.broken-inline-equation";

/// An inline equation at most this many body sizes wide is short.
const SHORT_EQUATION_EM: f64 = 6.0;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let items = &geometry.items;
    let body = crate::titles::body_size(geometry).unwrap_or(10.0);
    // The glyphs and shapes of each equation, in the order they are laid out.
    let mut equations: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, item) in items.iter().enumerate() {
        if !matches!(item.kind, ItemKind::Text | ItemKind::Shape) {
            continue;
        }
        let mut elem = item.elem;
        while let Some(current) = elem {
            if geometry.elements[current].name == "equation" {
                break;
            }
            elem = geometry.elements[current].parent;
        }
        if let Some(equation) = elem {
            equations.entry(equation).or_default().push(index);
        }
    }
    let mut out = Vec::new();
    for (equation, parts) in &equations {
        if parts.len() < 3 {
            continue;
        }
        let members: BTreeSet<usize> = parts.iter().copied().collect();
        for j in 0..parts.len() - 1 {
            let a = &items[parts[j]];
            let b = &items[parts[j + 1]];
            if a.page != b.page {
                continue;
            }
            // The next glyph starts left of where the equation began, a line
            // lower: the equation wraps.
            let line_height = a.bbox.height().max(6.0);
            let start = parts[..=j].iter().map(|&k| items[k].bbox.x0).fold(f64::INFINITY, f64::min);
            if b.bbox.x0 >= start - body || b.bbox.y0 < a.bbox.y1 - 0.2 * line_height {
                continue;
            }
            let first = extent(items, &parts[..=j]);
            let second = extent(items, &parts[j + 1..]);
            // Inline: running text before the first part and after the second.
            let before = text_on_row(items, &members, a, |o| o.bbox.x1 <= first.x0 + 1.0);
            let after = text_on_row(items, &members, b, |o| o.bbox.x0 >= second.x1 - 1.0);
            let width = first.width() + second.width();
            if before && after && width <= SHORT_EQUATION_EM * body {
                let ends = a.text.as_deref().unwrap_or_default().trim();
                let mut diagnostic = Diagnostic::new(
                    BROKEN_EQUATION,
                    Severity::Warning,
                    format!(
                        "a short inline equation on page {} breaks after \"{ends}\" and \
                         continues at the start of the next line, so its {}pt are torn apart",
                        a.page + 1,
                        round1(width)
                    ),
                );
                let span = geometry.elements[*equation].span;
                let span = if span.is_detached() { geometry.span_for(a.span, a.elem) } else { span };
                diagnostic.loc = diag::resolve_span(world, span);
                diagnostic.page = Some(a.page + 1);
                diagnostic.bbox = Some(first.union(&second).rounded());
                diagnostic.measured = Some(round1(width));
                diagnostic.limit = Some(round1(SHORT_EQUATION_EM * body));
                diagnostic.unit = Some("pt");
                diagnostic.hints.push(
                    "keep a short equation on one line: wrap it in a box, as in \
                     #box($36 div 6 = 6$), and set each step of a worked example on a line \
                     of its own"
                        .into(),
                );
                out.push(diagnostic);
            }
            break;
        }
    }
    out
}

/// Whether text outside the equation shares the row of `of` and passes
/// `test`.
fn text_on_row(
    items: &[Item],
    members: &BTreeSet<usize>,
    of: &Item,
    test: impl Fn(&Item) -> bool,
) -> bool {
    items.iter().enumerate().any(|(k, o)| {
        !members.contains(&k)
            && o.kind == ItemKind::Text
            && o.text.as_deref().is_some_and(|t| !t.trim().is_empty())
            && o.page == of.page
            && o.bbox.y0 < of.bbox.y1
            && o.bbox.y1 > of.bbox.y0
            && test(o)
    })
}

/// The box around the given items.
fn extent(items: &[Item], parts: &[usize]) -> BBox {
    parts.iter().map(|&k| items[k].bbox).reduce(|a, b| a.union(&b)).expect("parts are not empty")
}
