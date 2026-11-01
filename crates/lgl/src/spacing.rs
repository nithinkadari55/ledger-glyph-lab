use std::collections::HashMap;

use typst::syntax::Span;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::world::LabWorld;

pub const SPACING: &str = "L015.inconsistent-spacing";

/// Elements whose context changes the expected spacing of the text inside them.
const CONTEXTS: &[&str] =
    &["list", "enum", "terms", "table", "grid", "figure", "quote", "footnote", "outline"];

/// Gaps closer than this to the dominant gap are consistent.
const MIN_DEVIATION_PT: f64 = 1.5;
const MIN_DEVIATION_SHARE: f64 = 0.2;

/// How many gaps of a kind are needed before a dominant value is trusted.
const MIN_SAMPLES: usize = 3;

/// A paragraph or heading on one page, as the layout boxes of its lines.
struct Block {
    role: String,
    page: usize,
    bbox: BBox,
    span: Span,
}

struct Gap {
    key: String,
    size: f64,
    page: usize,
    span: Span,
    bbox: BBox,
}

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let blocks = blocks(geometry);
    let gaps = gaps(geometry, &blocks);

    let mut by_key: HashMap<&str, Vec<&Gap>> = HashMap::new();
    for gap in &gaps {
        by_key.entry(&gap.key).or_default().push(gap);
    }

    let mut out = Vec::new();
    for (key, gaps) in by_key {
        if gaps.len() < MIN_SAMPLES {
            continue;
        }
        let Some(dominant) = dominant(&gaps) else { continue };
        let tolerance = MIN_DEVIATION_PT.max(MIN_DEVIATION_SHARE * dominant);
        for gap in gaps.iter().filter(|g| (g.size - dominant).abs() > tolerance) {
            out.push(diagnostic(world, key, gap, dominant, tolerance));
        }
    }
    out.sort_by(|a, b| a.page.cmp(&b.page).then_with(|| a.message.cmp(&b.message)));
    out
}

fn blocks(geometry: &Geometry) -> Vec<Block> {
    let mut out: Vec<(Option<usize>, Block)> = Vec::new();
    for line in &geometry.lines {
        let Some(elem) = line.elem else { continue };
        let name = geometry.elements[elem].name;
        if name != "par" && name != "heading" {
            continue;
        }
        if let Some((last_elem, block)) = out.last_mut()
            && *last_elem == Some(elem)
            && block.page == line.page
            && overlaps_horizontally(&block.bbox, &line.bbox)
        {
            block.bbox = block.bbox.union(&line.bbox);
            continue;
        }
        let context = context(geometry, elem);
        out.push((
            Some(elem),
            Block {
                role: format!("{name}{context}"),
                page: line.page,
                bbox: line.bbox,
                span: geometry.elements[elem].span,
            },
        ));
    }
    out.into_iter().map(|(_, block)| block).collect()
}

fn context(geometry: &Geometry, elem: usize) -> String {
    let mut cursor = geometry.elements[elem].parent;
    while let Some(index) = cursor {
        let name = geometry.elements[index].name;
        if CONTEXTS.contains(&name) {
            return format!(" in {name}");
        }
        cursor = geometry.elements[index].parent;
    }
    String::new()
}

fn gaps(geometry: &Geometry, blocks: &[Block]) -> Vec<Gap> {
    let mut out = Vec::new();
    for pair in blocks.windows(2) {
        let (above, below) = (&pair[0], &pair[1]);
        if above.page != below.page || !overlaps_horizontally(&above.bbox, &below.bbox) {
            continue;
        }
        let size = below.bbox.y0 - above.bbox.y1;
        if size < 0.0 || something_between(geometry, above, below) {
            continue;
        }
        out.push(Gap {
            key: format!("{} -> {}", above.role, below.role),
            size,
            page: below.page,
            span: below.span,
            bbox: BBox { y0: above.bbox.y1, y1: below.bbox.y0, ..below.bbox },
        });
    }
    out
}

/// Whether visible content other than text lies between two blocks.
fn something_between(geometry: &Geometry, above: &Block, below: &Block) -> bool {
    let band = BBox {
        x0: above.bbox.x0.min(below.bbox.x0),
        y0: above.bbox.y1,
        x1: above.bbox.x1.max(below.bbox.x1),
        y1: below.bbox.y0,
    };
    geometry.items.iter().any(|item| {
        item.page == below.page
            && item.kind != ItemKind::Link
            && item.visible().and_then(|b| b.intersect(&band)).is_some_and(|b| {
                b.height() > MIN_DEVIATION_PT && b.width() > MIN_DEVIATION_PT
            })
    })
}

fn overlaps_horizontally(a: &BBox, b: &BBox) -> bool {
    a.x0 < b.x1 && b.x0 < a.x1
}

/// The most common gap size, in half-point bins, if it covers most gaps.
fn dominant(gaps: &[&Gap]) -> Option<f64> {
    let mut bins: HashMap<i64, Vec<f64>> = HashMap::new();
    for gap in gaps {
        bins.entry((gap.size * 2.0).round() as i64).or_default().push(gap.size);
    }
    let (_, sizes) = bins.into_iter().max_by(|a, b| {
        a.1.len().cmp(&b.1.len()).then_with(|| b.0.cmp(&a.0))
    })?;
    (sizes.len() * 2 > gaps.len()).then(|| sizes.iter().sum::<f64>() / sizes.len() as f64)
}

fn diagnostic(
    world: &LabWorld,
    key: &str,
    gap: &Gap,
    dominant: f64,
    tolerance: f64,
) -> Diagnostic {
    let size = round1(gap.size);
    let dominant = round1(dominant);
    let mut out = Diagnostic::new(
        SPACING,
        Severity::Warning,
        format!(
            "spacing {key} is {size}pt here but {dominant}pt elsewhere in the document \
             (page {})",
            gap.page + 1
        ),
    );
    out.loc = diag::resolve_span(world, gap.span);
    out.page = Some(gap.page + 1);
    out.bbox = Some(gap.bbox.rounded());
    out.measured = Some(size);
    out.limit = Some(round1(tolerance));
    out.unit = Some("pt");
    out.hints.push(if gap.size > dominant {
        "remove manual vertical space (v, linebreaks, empty paragraphs) before this \
         content, or set spacing once with set rules"
            .into()
    } else {
        "avoid negative or reduced manual spacing here; set spacing once with set rules"
            .into()
    });
    out
}
