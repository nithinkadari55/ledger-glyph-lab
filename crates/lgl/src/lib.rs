pub mod boxes;
pub mod catalog;
pub mod diag;
pub mod fix;
pub mod geom;
pub mod grid;
pub mod lint;
pub mod overlap;
pub mod pagecheck;
pub mod quality;
pub mod resources;
pub mod rhythm;
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
    if let Ok(source) = typst::World::source(world, typst::World::main(world)) {
        diagnostics.extend(lint::run(world, &source));
    }
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

/// Parses the main file without compiling it and reports its syntax errors.
/// This is fast enough to run after every edit.
pub fn syntax(world: &LabWorld) -> Vec<Diagnostic> {
    let Ok(source) = typst::World::source(world, typst::World::main(world)) else {
        return Vec::new();
    };
    let (errors, _) = source.root().errors_and_warnings();
    errors
        .iter()
        .map(|error| {
            let mut out =
                Diagnostic::new("typst.syntax.error", Severity::Error, error.message.as_str());
            out.loc = diag::resolve(world, error.span);
            out.hints = error.hints.iter().map(|hint| hint.v.to_string()).collect();
            out
        })
        .collect()
}

/// The font families available to documents: the embedded fonts plus any in
/// `font_dirs`, sorted by name.
pub fn font_families(font_dirs: &[std::path::PathBuf]) -> Vec<String> {
    let mut fonts = typst_kit::fonts::FontStore::new();
    fonts.extend(typst_kit::fonts::embedded());
    for dir in font_dirs {
        fonts.extend(typst_kit::fonts::scan(dir));
    }
    let mut families: Vec<String> =
        fonts.book().families().map(|(name, _)| name.to_string()).collect();
    families.sort();
    families.dedup();
    families
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
