use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, Line};
use crate::world::LabWorld;

pub const ORPHAN: &str = "L010.orphan";
pub const WIDOW: &str = "L011.widow";
pub const HYPHEN_LADDER: &str = "L012.hyphen-ladder";
pub const HYPHEN_AT_BREAK: &str = "L013.hyphen-at-page-break";
pub const STRANDED_HEADING: &str = "L014.stranded-heading";

/// The most consecutive lines that may end in an inserted hyphen. Convention.
const MAX_HYPHEN_RUN: usize = 2;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for lines in paragraphs(geometry) {
        breaks(world, &lines, &mut out);
        hyphen_ladder(world, &lines, &mut out);
    }
    stranded_headings(world, geometry, &mut out);
    out
}

/// The lines of each paragraph in document order. Lines that do not belong to
/// a paragraph element are skipped.
fn paragraphs(geometry: &Geometry) -> Vec<Vec<&Line>> {
    let mut out: Vec<Vec<&Line>> = Vec::new();
    let mut current = None;
    for line in &geometry.lines {
        let Some(elem) = line.elem.filter(|&e| geometry.elements[e].name == "par") else {
            current = None;
            continue;
        };
        if current != Some(elem) {
            out.push(Vec::new());
            current = Some(elem);
        }
        out.last_mut().unwrap().push(line);
    }
    out
}

fn breaks(world: &LabWorld, lines: &[&Line], out: &mut Vec<Diagnostic>) {
    for pair in lines.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        if before.page == after.page {
            continue;
        }
        let on_before = lines.iter().filter(|l| l.page == before.page).count();
        let on_after = lines.iter().filter(|l| l.page == after.page).count();
        if on_before == 1 && lines.len() > 1 {
            out.push(line_diagnostic(
                world,
                ORPHAN,
                before,
                format!(
                    "the first line of a paragraph is left alone at the bottom of page {}",
                    before.page + 1
                ),
                "keep at least two lines together (the default par costs prevent this \
                 unless overridden), or adjust the preceding content",
            ));
        }
        if on_after == 1 && lines.len() > 1 {
            out.push(line_diagnostic(
                world,
                WIDOW,
                after,
                format!(
                    "the last line of a paragraph is left alone at the top of page {}",
                    after.page + 1
                ),
                "keep at least two lines together (the default par costs prevent this \
                 unless overridden), or adjust the paragraph length",
            ));
        }
        if before.hyphenated {
            out.push(line_diagnostic(
                world,
                HYPHEN_AT_BREAK,
                before,
                format!(
                    "a hyphenated word is split across pages {} and {}",
                    before.page + 1,
                    after.page + 1
                ),
                "rephrase the line or adjust the preceding content so the word is not \
                 split across the page break",
            ));
        }
    }
}

fn hyphen_ladder(world: &LabWorld, lines: &[&Line], out: &mut Vec<Diagnostic>) {
    let mut run = 0;
    for (i, line) in lines.iter().enumerate() {
        run = if line.hyphenated { run + 1 } else { 0 };
        let ends_run = !lines.get(i + 1).is_some_and(|next| next.hyphenated);
        if run > MAX_HYPHEN_RUN && ends_run {
            let first = lines[i + 1 - run];
            let mut diagnostic = line_diagnostic(
                world,
                HYPHEN_LADDER,
                first,
                format!(
                    "{run} consecutive lines end with a hyphen on page {}",
                    first.page + 1
                ),
                "rephrase the paragraph or adjust the text width so fewer consecutive \
                 lines are hyphenated",
            );
            diagnostic.measured = Some(run as f64);
            diagnostic.limit = Some(MAX_HYPHEN_RUN as f64);
            diagnostic.unit = Some("lines");
            out.push(diagnostic);
        }
    }
}

fn stranded_headings(world: &LabWorld, geometry: &Geometry, out: &mut Vec<Diagnostic>) {
    for (index, element) in geometry.elements.iter().enumerate() {
        if element.name != "heading" {
            continue;
        }
        let Some(last) = element.boxes.last() else { continue };
        let page = last.page;
        let below = geometry.items.iter().any(|item| {
            item.page == page
                && geometry.is_body(item.region)
                && item.bbox.y0 >= last.bbox.y1
                && !descends_from(geometry, item.elem, index)
        });
        let continues = geometry.items.iter().any(|item| item.page > page);
        if below || !continues {
            continue;
        }
        let mut diagnostic = Diagnostic::new(
            STRANDED_HEADING,
            Severity::Warning,
            format!(
                "heading is the last content on page {}; its section starts on the next page",
                page + 1
            ),
        );
        diagnostic.loc = diag::resolve_span(world, element.span);
        diagnostic.elem = Some("heading".into());
        diagnostic.label = element.label.clone();
        diagnostic.page = Some(page + 1);
        diagnostic.bbox = Some(last.bbox.rounded());
        diagnostic.hints.push(
            "keep headings with the following content (do not disable the default \
             sticky behaviour), or adjust the preceding content"
                .into(),
        );
        out.push(diagnostic);
    }
}

fn descends_from(
    geometry: &Geometry,
    mut cursor: Option<usize>,
    ancestor: usize,
) -> bool {
    while let Some(index) = cursor {
        if index == ancestor {
            return true;
        }
        cursor = geometry.elements[index].parent;
    }
    false
}

fn line_diagnostic(
    world: &LabWorld,
    rule: &'static str,
    line: &Line,
    message: String,
    hint: &str,
) -> Diagnostic {
    let mut out = Diagnostic::new(rule, Severity::Warning, message);
    out.loc = diag::resolve_span(world, line.span);
    out.elem = Some("par".into());
    out.page = Some(line.page + 1);
    out.bbox = Some(line.bbox.rounded());
    out.hints.push(hint.into());
    out
}
