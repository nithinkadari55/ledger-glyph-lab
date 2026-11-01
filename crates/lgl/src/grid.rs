use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, Line, round1};
use crate::world::LabWorld;

pub const OFF_GRID: &str = "L030.off-baseline-grid";

/// How far a baseline may be from its grid line.
const TOLERANCE_PT: f64 = 0.05;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let mut pages: BTreeMap<usize, (usize, f64, &Line, f64)> = BTreeMap::new();
    for line in &geometry.lines {
        let Some(region) = line.region.map(|r| &geometry.regions[r]) else { continue };
        let Some(pitch) = region.grid else { continue };
        if region.kind != "page-body"
            || line.container.is_some()
            || !is_running_text(geometry, line)
        {
            continue;
        }
        let offset = (line.bbox.y0 + line.baseline - region.bbox.y0).rem_euclid(pitch);
        let deviation = offset.min(pitch - offset);
        if deviation <= TOLERANCE_PT {
            continue;
        }
        let entry = pages.entry(line.page).or_insert((0, 0.0, line, pitch));
        entry.0 += 1;
        if deviation > entry.1 {
            entry.1 = deviation;
        }
    }

    pages
        .into_iter()
        .map(|(page, (count, worst, first, pitch))| {
            let mut out = Diagnostic::new(
                OFF_GRID,
                Severity::Error,
                format!(
                    "{count} line(s) on page {} sit off the {pitch}pt baseline grid, by up \
                     to {}pt",
                    page + 1,
                    round1(worst)
                ),
            );
            out.loc = diag::resolve_span(world, first.span);
            out.page = Some(page + 1);
            out.bbox = Some(first.bbox.rounded());
            out.measured = Some(round1(worst));
            out.limit = Some(TOLERANCE_PT);
            out.unit = Some("pt");
            out.hints.push(
                "avoid fractional spacing (v(1fr)) and bottom alignment around grid text, \
                 or make the spacing a multiple of the grid pitch"
                    .into(),
            );
            out
        })
        .collect()
}

/// Elements whose content is laid out in a nested flow, which is not snapped
/// to the grid.
const NESTED: &[&str] =
    &["entry", "list", "enum", "terms", "table", "grid", "figure", "quote", "outline"];

/// Lines of top-level paragraphs.
fn is_running_text(geometry: &Geometry, line: &Line) -> bool {
    let mut cursor = line.elem;
    let mut first = true;
    while let Some(index) = cursor {
        let name = geometry.elements[index].name;
        if NESTED.contains(&name) || (first && name != "par") {
            return false;
        }
        first = false;
        cursor = geometry.elements[index].parent;
    }
    true
}
