pub mod catalog;
pub mod diag;
pub mod fix;
pub mod geom;
pub mod grid;
pub mod overlap;
pub mod pagecheck;
pub mod quality;
pub mod resources;
pub mod rules;
pub mod spacing;
pub mod structure;
pub mod typography;
pub mod whitespace;
pub mod world;

use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

use crate::diag::{Diagnostic, Severity};
use crate::geom::Geometry;
use crate::world::LabWorld;

pub struct Outcome {
    pub diagnostics: Vec<Diagnostic>,
    pub geometry: Option<Geometry>,
    pub pdf: Option<Vec<u8>>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        !self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
}

pub fn check(world: &LabWorld, export_pdf: bool) -> Outcome {
    let mut diagnostics = Vec::new();
    let compiled = typst::compile::<PagedDocument>(world);
    diagnostics.extend(
        compiled
            .warnings
            .iter()
            .map(|w| diag::from_typst(world, w, "compile")),
    );

    let document = match compiled.output {
        Ok(document) => document,
        Err(errors) => {
            diagnostics
                .extend(errors.iter().map(|e| diag::from_typst(world, e, "compile")));
            return Outcome { diagnostics, geometry: None, pdf: None };
        }
    };

    let geometry = geom::extract(&document);
    diagnostics.extend(rules::run(world, &document, &geometry));

    let pdf = if export_pdf {
        let exported = typst_pdf::pdf(&document, &PdfOptions::default());
        diagnostics
            .extend(exported.warnings.iter().map(|w| diag::from_typst(world, w, "pdf")));
        match exported.output {
            Ok(bytes) => Some(bytes),
            Err(errors) => {
                diagnostics
                    .extend(errors.iter().map(|e| diag::from_typst(world, e, "pdf")));
                None
            }
        }
    } else {
        None
    };

    fix::suggest(world, &mut diagnostics);

    diagnostics.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.page.cmp(&b.page))
            .then_with(|| a.rule.cmp(&b.rule))
    });

    Outcome { diagnostics, geometry: Some(geometry), pdf }
}

/// Repeatedly applies the suggested fixes for the main file, starting from
/// `text`, and writes each version to `out`. The world is rooted next to
/// `out`, so `out` must be in the same directory as the original file.
///
/// Returns the number of applied edits and rounds.
pub fn repair(
    text: &str,
    out: &std::path::Path,
    options: &world::WorldOptions,
    max_rounds: usize,
) -> Result<(usize, usize), String> {
    let mut text = text.to_string();
    let mut applied_total = 0;
    let mut rounds = 0;
    loop {
        std::fs::write(out, &text).map_err(|err| format!("{}: {err}", out.display()))?;
        if rounds == max_rounds {
            break;
        }
        let world = LabWorld::new(out, options).map_err(|err| err.to_string())?;
        let outcome = check(&world, false);
        let main = world.display_name(typst::World::main(&world));
        let edits: Vec<&fix::Edit> = outcome
            .diagnostics
            .iter()
            .filter_map(|d| d.fix.as_ref())
            .flat_map(|f| &f.edits)
            .filter(|e| e.loc.file == main)
            .collect();
        if edits.is_empty() {
            break;
        }
        let (fixed, applied) = fix::apply(&text, &edits);
        if applied == 0 || fixed == text {
            break;
        }
        text = fixed;
        applied_total += applied;
        rounds += 1;
    }
    Ok((applied_total, rounds))
}
