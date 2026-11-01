use serde::Serialize;
use typst::WorldExt;
use typst::diag::{Severity as TypstSeverity, SourceDiagnostic};
use typst::syntax::{DiagSpan, Span};

use crate::geom::BBox;
use crate::world::LabWorld;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLoc {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub v: u32,
    pub rule: String,
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc: Option<SourceLoc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elem: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<BBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub measured: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trace: Vec<TraceEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TraceEntry {
    pub note: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc: Option<SourceLoc>,
}

impl Diagnostic {
    pub fn new(rule: impl Into<String>, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            v: SCHEMA_VERSION,
            rule: rule.into(),
            severity,
            message: message.into(),
            loc: None,
            elem: None,
            label: None,
            page: None,
            bbox: None,
            measured: None,
            limit: None,
            unit: None,
            hints: Vec::new(),
            trace: Vec::new(),
        }
    }
}

pub fn resolve(world: &LabWorld, span: impl Into<DiagSpan>) -> Option<SourceLoc> {
    let span = span.into();
    let id = span.id()?;
    let range = world.range(span)?;
    let source = typst::World::source(world, id).ok()?;
    let lines = source.lines();
    let (line, col) = lines.byte_to_line_column(range.start)?;
    let (end_line, end_col) = lines.byte_to_line_column(range.end)?;
    Some(SourceLoc {
        file: world.display_name(id),
        line: line + 1,
        col: col + 1,
        end_line: end_line + 1,
        end_col: end_col + 1,
    })
}

pub fn resolve_span(world: &LabWorld, span: Span) -> Option<SourceLoc> {
    if span.is_detached() { None } else { resolve(world, span) }
}

pub fn from_typst(world: &LabWorld, diag: &SourceDiagnostic, stage: &str) -> Diagnostic {
    let severity = match diag.severity {
        TypstSeverity::Error => Severity::Error,
        TypstSeverity::Warning => Severity::Warning,
    };
    let kind = match severity {
        Severity::Error => "error",
        Severity::Warning | Severity::Info => "warning",
    };
    let mut out = Diagnostic::new(format!("typst.{stage}.{kind}"), severity, diag.message.as_str());
    out.loc = resolve(world, diag.span);
    out.hints = diag
        .hints
        .iter()
        .map(|hint| match resolve(world, hint.span) {
            Some(loc) => format!("{} ({}:{}:{})", hint.v, loc.file, loc.line, loc.col),
            None => hint.v.to_string(),
        })
        .collect();
    out.trace = diag
        .trace
        .iter()
        .map(|point| TraceEntry {
            note: point.v.to_string(),
            loc: resolve_span(world, point.span),
        })
        .collect();
    out
}
