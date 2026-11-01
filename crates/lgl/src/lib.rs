pub mod diag;
pub mod geom;
pub mod overlap;
pub mod resources;
pub mod rules;
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

    diagnostics.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.page.cmp(&b.page))
            .then_with(|| a.rule.cmp(&b.rule))
    });

    Outcome { diagnostics, geometry: Some(geometry), pdf }
}
