pub mod boxes;
pub mod catalog;
pub mod composition;
pub mod contrast;
pub mod diag;
pub mod fix;
pub mod geom;
pub mod grid;
pub mod lint;
pub mod marks;
pub mod overlap;
pub mod pagecheck;
pub mod quality;
pub mod resources;
pub mod rhythm;
pub mod rules;
pub mod spacing;
pub mod structure;
pub mod titles;
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
            let mut out = Diagnostic::new(
                "typst.syntax.error",
                Severity::Error,
                error.message.as_str(),
            );
            out.loc = diag::resolve(world, error.span);
            out.hints = error.hints.iter().map(|hint| hint.v.to_string()).collect();
            out
        })
        .collect()
}

/// The font families available to documents, sorted by name.
pub fn font_families(font_dirs: &[std::path::PathBuf]) -> Vec<FamilyInfo> {
    use typst::text::{FontFlags, FontStyle, Tag};

    let fonts = world::font_store(font_dirs);
    let book = fonts.book();
    let mut families: Vec<FamilyInfo> = book
        .families()
        .map(|(name, ids)| {
            let mut family = FamilyInfo {
                family: name.to_string(),
                weights: [u16::MAX, 0],
                italic: false,
                axes: Vec::new(),
                math: false,
                monospace: false,
                smallcaps: false,
            };
            for id in ids {
                let Some(info) = book.info(id) else { continue };
                let weight = info.variant.weight.to_number();
                let (mut light, mut heavy) = (weight, weight);
                for axis in &info.axes {
                    if axis.tag == Tag::from_bytes(b"wght") {
                        light = axis.min.0 as u16;
                        heavy = axis.max.0 as u16;
                    } else if !family.axes.contains(&axis.tag.to_string()) {
                        family.axes.push(axis.tag.to_string());
                    }
                }
                let [lightest, heaviest] = family.weights;
                family.weights = [lightest.min(light), heaviest.max(heavy)];
                family.italic |= info.variant.style != FontStyle::Normal;
                family.math |= info.flags.contains(FontFlags::MATH);
                family.monospace |= info.flags.contains(FontFlags::MONOSPACE);
                family.smallcaps |= fonts.font(id).is_some_and(|font| has_smallcaps(&font));
            }
            family.axes.sort();
            family
        })
        .collect();
    families.sort_by(|a, b| a.family.cmp(&b.family));
    families.dedup_by(|a, b| a.family == b.family);
    families
}

/// A font family available to documents.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FamilyInfo {
    pub family: String,
    /// The lightest and the heaviest weight.
    pub weights: [u16; 2],
    /// Whether the family has italic or oblique styles.
    pub italic: bool,
    /// Variation axes other than weight, like `wdth` (set with `stretch`) or
    /// `opsz`.
    pub axes: Vec<String>,
    /// Whether the family can typeset math.
    pub math: bool,
    pub monospace: bool,
    /// Whether the family has true small capitals (the `smcp` feature), which
    /// `smallcaps()` needs.
    pub smallcaps: bool,
}

/// Whether the font's substitution table offers small capitals.
fn has_smallcaps(font: &typst::text::Font) -> bool {
    font.ttf()
        .tables()
        .gsub
        .is_some_and(|gsub| gsub.features.into_iter().any(|f| f.tag.to_bytes() == *b"smcp"))
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
