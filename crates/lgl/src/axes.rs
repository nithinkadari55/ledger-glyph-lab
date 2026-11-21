//! The axis of a timeline or a process line: a line drawn in pieces (one per
//! grid cell) with a piece off the level of the others, and event lines that
//! stop short of the dot they belong to. Measured in batch 21 on the stored
//! documents: 5 timelines with a lifted last piece (rounds 41 and 42 among
//! them) and 5 whose event lines start 6 to 13pt away from their dots.

use std::collections::BTreeMap;

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{Geometry, Item, ItemKind, round1};
use crate::world::LabWorld;

pub const BROKEN_AXIS: &str = "L102.broken-axis";

/// Axis pieces: thinner than this and at least this long, in points.
const PIECE_MAX_HEIGHT_PT: f64 = 4.0;
const PIECE_MIN_WIDTH_PT: f64 = 25.0;

/// Consecutive pieces of one axis meet within this distance, in points.
const PIECE_JOIN_PT: f64 = 14.0;

/// An axis has at least this many pieces before its level is judged.
const MIN_PIECES: usize = 4;

/// A piece this far off the level of the others is stepped, in points.
const STEP_PT: std::ops::RangeInclusive<f64> = 1.0..=8.0;

/// Dots: round marks of this size, in points.
const DOT_PT: std::ops::RangeInclusive<f64> = 4.0..=16.0;

/// Event lines: thinner than 3pt and at least 20pt long. One stops short of
/// its dot when the gap is in this range; a gap of 2pt (a deliberate hair
/// gap) read as joined in an approved timeline.
const STEM_GAP_PT: std::ops::RangeInclusive<f64> = 4.0..=30.0;

/// Event lines that stop short, per page, before it counts.
const MIN_SHORT_STEMS: usize = 2;

fn width(item: &Item) -> f64 {
    item.bbox.width()
}

fn height(item: &Item) -> f64 {
    item.bbox.height()
}

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let body = crate::titles::body_size(geometry).unwrap_or(10.0);
    let mut pages: BTreeMap<usize, Vec<&Item>> = BTreeMap::new();
    for item in &geometry.items {
        if item.kind == ItemKind::Shape {
            pages.entry(item.page).or_default().push(item);
        }
    }
    let mut out = Vec::new();
    for (page, shapes) in pages {
        out.extend(stepped_axis(world, geometry, page, &shapes));
        out.extend(short_stems(world, geometry, page, &shapes));
        out.extend(long_stems(world, geometry, page, &shapes, body));
    }
    out
}

/// Chains of axis pieces of one thickness and colour, each starting within a
/// short distance of the previous one's end, where a piece sits off the level
/// that most pieces share.
fn stepped_axis(
    world: &LabWorld,
    geometry: &Geometry,
    page: usize,
    shapes: &[&Item],
) -> Option<Diagnostic> {
    let mut groups: BTreeMap<(i64, String), Vec<&Item>> = BTreeMap::new();
    for shape in shapes {
        if height(shape) < PIECE_MAX_HEIGHT_PT && width(shape) >= PIECE_MIN_WIDTH_PT {
            let key =
                ((height(shape) * 10.0).round() as i64, format!("{:?}", shape.color));
            groups.entry(key).or_default().push(shape);
        }
    }
    for pieces in groups.values_mut() {
        pieces.sort_by(|a, b| a.bbox.x0.total_cmp(&b.bbox.x0));
        let mut chains: Vec<Vec<&Item>> = Vec::new();
        for piece in pieces.iter() {
            match chains.last_mut() {
                Some(chain)
                    if (0.0..=PIECE_JOIN_PT)
                        .contains(&(piece.bbox.x0 - chain.last().unwrap().bbox.x1))
                        && (piece.bbox.y0 - chain.last().unwrap().bbox.y0).abs()
                            <= *STEP_PT.end() =>
                {
                    chain.push(piece)
                }
                _ => chains.push(vec![piece]),
            }
        }
        for chain in chains.iter().filter(|c| c.len() >= MIN_PIECES) {
            let mut levels: BTreeMap<i64, usize> = BTreeMap::new();
            for piece in chain {
                *levels.entry((piece.bbox.y0 * 10.0).round() as i64).or_default() += 1;
            }
            let (&main, _) = levels.iter().max_by_key(|(_, n)| **n)?;
            let level = main as f64 / 10.0;
            let Some(odd) =
                chain.iter().find(|p| STEP_PT.contains(&(p.bbox.y0 - level).abs()))
            else {
                continue;
            };
            let step = odd.bbox.y0 - level;
            let mut diagnostic = Diagnostic::new(
                BROKEN_AXIS,
                Severity::Warning,
                format!(
                    "the axis on page {} is drawn in {} pieces and one sits {}pt {} the \
                     others, so the line has a step",
                    page + 1,
                    chain.len(),
                    round1(step.abs()),
                    if step < 0.0 { "above" } else { "below" }
                ),
            );
            diagnostic.loc =
                diag::resolve_span(world, geometry.span_for(odd.span, odd.elem));
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = Some(
                chain
                    .iter()
                    .skip(1)
                    .fold(chain[0].bbox, |acc, p| acc.union(&p.bbox))
                    .rounded(),
            );
            diagnostic.measured = Some(round1(step.abs()));
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "draw the axis as one line across the whole width (place(line(length: \
                 100%)) behind the events, or a single line before the grid), not one \
                 piece per grid cell"
                    .into(),
            );
            return Some(diagnostic);
        }
    }
    None
}

/// Event lines (stems) that point at a dot on the axis but stop short of it.
fn short_stems(
    world: &LabWorld,
    geometry: &Geometry,
    page: usize,
    shapes: &[&Item],
) -> Option<Diagnostic> {
    let dots: Vec<&Item> = shapes
        .iter()
        .copied()
        .filter(|s| DOT_PT.contains(&width(s)) && (width(s) - height(s)).abs() <= 1.0)
        .collect();
    let verticals: Vec<&Item> =
        shapes.iter().copied().filter(|s| width(s) < 3.0).collect();
    let mut short: Vec<(&Item, f64)> = Vec::new();
    for stem in verticals.iter().filter(|s| height(s) >= 20.0) {
        let x = (stem.bbox.x0 + stem.bbox.x1) / 2.0;
        let gap = |dot: &Item| {
            if stem.bbox.y0 >= dot.bbox.y1 {
                stem.bbox.y0 - dot.bbox.y1
            } else if stem.bbox.y1 <= dot.bbox.y0 {
                dot.bbox.y0 - stem.bbox.y1
            } else {
                0.0
            }
        };
        let Some(dot) = dots
            .iter()
            .filter(|d| d.bbox.x0 - 0.5 <= x && x <= d.bbox.x1 + 0.5)
            .min_by(|a, b| gap(a).total_cmp(&gap(b)))
        else {
            continue;
        };
        let distance = gap(dot);
        if !STEM_GAP_PT.contains(&distance) {
            continue;
        }
        // Another piece of the same line may bridge the gap.
        let (low, high) = if stem.bbox.y0 >= dot.bbox.y1 {
            (dot.bbox.y1, stem.bbox.y0)
        } else {
            (stem.bbox.y1, dot.bbox.y0)
        };
        let bridged = verticals.iter().any(|v| {
            !std::ptr::eq(*v, *stem)
                && ((v.bbox.x0 + v.bbox.x1) / 2.0 - x).abs() <= 1.0
                && v.bbox.y0 <= low + 1.0
                && v.bbox.y1 >= high - 1.0
        });
        if !bridged {
            short.push((stem, distance));
        }
    }
    if short.len() < MIN_SHORT_STEMS {
        return None;
    }
    let (first, distance) = short[0];
    let mut diagnostic = Diagnostic::new(
        BROKEN_AXIS,
        Severity::Warning,
        format!(
            "{} event lines on page {} stop {}pt short of the dot they point at, so \
             they do not connect to the axis",
            short.len(),
            page + 1,
            round1(distance)
        ),
    );
    diagnostic.loc = diag::resolve_span(world, geometry.span_for(first.span, first.elem));
    diagnostic.page = Some(page + 1);
    diagnostic.bbox = Some(first.bbox.rounded());
    diagnostic.measured = Some(round1(distance));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "let every event line run to the centre of its dot: start lines above the axis \
         at the dot and lines below the axis at the dot too, with the same offset on \
         both sides"
            .into(),
    );
    Some(diagnostic)
}

/// An event line runs this many body sizes past the text beside it before it
/// counts. Measured in batch 22: the three stored timelines whose lines are
/// borders of fixed-height cells run them 47 to 96pt past their text; the
/// approved timelines end their lines at the text.
const STEM_OVERRUN_EM: f64 = 2.5;

/// Event lines that start at a dot and run far past the text they mark (the
/// border of a fixed-height cell instead of a line of its own).
fn long_stems(
    world: &LabWorld,
    geometry: &Geometry,
    page: usize,
    shapes: &[&Item],
    body: f64,
) -> Option<Diagnostic> {
    let dots: Vec<&Item> = shapes
        .iter()
        .copied()
        .filter(|s| DOT_PT.contains(&width(s)) && (width(s) - height(s)).abs() <= 1.0)
        .collect();
    if dots.len() < 3 {
        return None;
    }
    let texts: Vec<&Item> = geometry
        .items
        .iter()
        .filter(|i| {
            i.page == page
                && i.kind == ItemKind::Text
                && i.text.as_deref().is_some_and(|t| !t.trim().is_empty())
        })
        .collect();
    let mut long: Vec<(&Item, f64)> = Vec::new();
    for stem in shapes.iter().filter(|s| width(s) < 3.0 && height(s) >= 30.0) {
        let b = stem.bbox;
        let x = (b.x0 + b.x1) / 2.0;
        let Some(dot) = dots.iter().find(|d| {
            d.bbox.x0 - 2.0 <= x
                && x <= d.bbox.x1 + 2.0
                && ((d.bbox.y1 - b.y0).abs() <= 8.0
                    || (d.bbox.y0 - b.y1).abs() <= 8.0
                    || (d.bbox.y0..=d.bbox.y1).contains(&b.y0)
                    || (d.bbox.y0..=d.bbox.y1).contains(&b.y1))
        }) else {
            continue;
        };
        let beside: Vec<&&Item> = texts
            .iter()
            .filter(|t| {
                (0.0..=40.0).contains(&(t.bbox.x0 - x))
                    && t.bbox.y1 > b.y0
                    && t.bbox.y0 < b.y1
            })
            .collect();
        if beside.is_empty() {
            continue;
        }
        let below = b.y0 >= dot.bbox.y0 - 8.0;
        let overrun = if below {
            b.y1 - beside.iter().map(|t| t.bbox.y1).fold(f64::NEG_INFINITY, f64::max)
        } else {
            beside.iter().map(|t| t.bbox.y0).fold(f64::INFINITY, f64::min) - b.y0
        };
        if overrun > STEM_OVERRUN_EM * body {
            long.push((stem, overrun));
        }
    }
    if long.len() < MIN_SHORT_STEMS {
        return None;
    }
    let (first, overrun) = long[0];
    let mut diagnostic = Diagnostic::new(
        BROKEN_AXIS,
        Severity::Warning,
        format!(
            "{} event lines on page {} run {}pt past the text they mark, so they reach \
             into empty space",
            long.len(),
            page + 1,
            round1(overrun)
        ),
    );
    diagnostic.loc = diag::resolve_span(world, geometry.span_for(first.span, first.elem));
    diagnostic.page = Some(page + 1);
    diagnostic.bbox = Some(first.bbox.rounded());
    diagnostic.measured = Some(round1(overrun));
    diagnostic.limit = Some(round1(STEM_OVERRUN_EM * body));
    diagnostic.unit = Some("pt");
    diagnostic.hints.push(
        "draw each event line from its text to its dot, as a line of its own, not as the \
         border of a fixed-height cell; let the rows take their content's height"
            .into(),
    );
    Some(diagnostic)
}
