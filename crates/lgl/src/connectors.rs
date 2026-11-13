//! Connector lines of a diagram that end in the air. Ported unchanged from
//! batch 8/9 (L092), where it was measured on 394 generated documents.

use crate::diag::{self, Diagnostic, Severity};
use crate::geom::{BBox, Geometry, ItemKind, round1};
use crate::rules::line_thickness;
use crate::world::LabWorld;

pub const BROKEN_CONNECTOR: &str = "L092.broken-connector";

/// Lines and shapes at most this far apart touch.
const TOUCH_PT: f64 = 1.0;

/// A shape at least this large on both sides is a box or a marker that a
/// line can join; a smaller one is a speck.
const MIN_MARK_PT: f64 = 3.0;

/// The end of a line this close to text is a leader to that text.
const LABEL_REACH_PT: f64 = 3.0;

/// Connectors of one diagram are drawn alike: their thickness differs by at
/// most this.
const SAME_STROKE_PT: f64 = 0.2;

/// A page holds a diagram when at least this many lines join a box at one
/// end and a box or a line at the other.
const MIN_CONNECTORS: usize = 2;

/// The loose end of a line stops short of a box when it is at most this many
/// body text sizes from it. Measured on 394 generated documents: the broken
/// connectors of six organisation charts and one timeline stop 0.3 to 1.9
/// sizes short; the nearest box that a loose end is not meant to reach is 4
/// sizes away.
const MAX_CONNECTOR_GAP_EM: f64 = 2.0;

/// A target may lie this far behind the loose end, along the line.
const BEHIND_PT: f64 = 2.0;

pub fn run(world: &LabWorld, geometry: &Geometry) -> Vec<Diagnostic> {
    let Some(body) = crate::titles::body_size(geometry) else { return Vec::new() };
    broken_connectors(world, geometry, body)
}

/// A straight line of a drawing: one thin shape, or several end to end.
struct Stroke {
    vertical: bool,
    /// Where the line lies across its direction, and where it starts and ends
    /// along it.
    at: f64,
    start: f64,
    end: f64,
    thickness: f64,
    item: usize,
}

impl Stroke {
    fn bbox(&self) -> BBox {
        let half = self.thickness / 2.0;
        if self.vertical {
            BBox {
                x0: self.at - half,
                y0: self.start,
                x1: self.at + half,
                y1: self.end,
            }
        } else {
            BBox {
                x0: self.start,
                y0: self.at - half,
                x1: self.end,
                y1: self.at + half,
            }
        }
    }

    /// The point at the first or the last end.
    fn tip(&self, last: bool) -> (f64, f64) {
        let along = if last { self.end } else { self.start };
        if self.vertical { (self.at, along) } else { (along, self.at) }
    }
}

fn grown(b: &BBox, by: f64) -> BBox {
    BBox {
        x0: b.x0 - by,
        y0: b.y0 - by,
        x1: b.x1 + by,
        y1: b.y1 + by,
    }
}

fn holds(b: &BBox, (x, y): (f64, f64)) -> bool {
    b.x0 <= x && x <= b.x1 && b.y0 <= y && y <= b.y1
}

fn meet(a: &BBox, b: &BBox) -> bool {
    a.x0 <= b.x1 + TOUCH_PT
        && b.x0 <= a.x1 + TOUCH_PT
        && a.y0 <= b.y1 + TOUCH_PT
        && b.y0 <= a.y1 + TOUCH_PT
}

fn encloses(outer: &BBox, inner: &BBox) -> bool {
    outer.x0 <= inner.x0 + 0.5
        && outer.y0 <= inner.y0 + 0.5
        && inner.x1 <= outer.x1 + 0.5
        && inner.y1 <= outer.y1 + 0.5
}

fn nearest(b: &BBox, (x, y): (f64, f64)) -> (f64, f64) {
    (x.clamp(b.x0, b.x1), y.clamp(b.y0, b.y1))
}

fn distance(b: &BBox, point: (f64, f64)) -> f64 {
    let (x, y) = nearest(b, point);
    let (dx, dy) = (x - point.0, y - point.1);
    (dx * dx + dy * dy).sqrt()
}

fn in_table(geometry: &Geometry, mut elem: Option<usize>) -> bool {
    while let Some(index) = elem {
        let element = &geometry.elements[index];
        if element.name == "table" {
            return true;
        }
        elem = element.parent;
    }
    false
}

/// The straight lines drawn on a page outside tables, with pieces that
/// continue each other joined into one line.
fn strokes(geometry: &Geometry, page: usize) -> Vec<Stroke> {
    let mut pieces: Vec<Stroke> = geometry
        .items
        .iter()
        .enumerate()
        .filter(|(_, i)| {
            i.page == page && i.kind == ItemKind::Shape && !in_table(geometry, i.elem)
        })
        .filter_map(|(item, i)| {
            let b = &i.bbox;
            let thickness = line_thickness(b)?;
            let vertical = b.height() > b.width();
            let (at, start, end) = if vertical {
                ((b.x0 + b.x1) / 2.0, b.y0, b.y1)
            } else {
                ((b.y0 + b.y1) / 2.0, b.x0, b.x1)
            };
            Some(Stroke { vertical, at, start, end, thickness, item })
        })
        .collect();
    let bin = |s: &Stroke| (s.at * 2.0).round() as i64;
    pieces.sort_by(|a, b| {
        a.vertical
            .cmp(&b.vertical)
            .then(bin(a).cmp(&bin(b)))
            .then(a.start.total_cmp(&b.start))
    });
    let mut out: Vec<Stroke> = Vec::new();
    for piece in pieces {
        match out.last_mut() {
            Some(last)
                if last.vertical == piece.vertical
                    && (last.at - piece.at).abs() <= 0.5
                    && (last.thickness - piece.thickness).abs() <= 0.3
                    && piece.start <= last.end + TOUCH_PT =>
            {
                last.end = last.end.max(piece.end)
            }
            _ => out.push(piece),
        }
    }
    out
}

fn root(parent: &mut [usize], mut node: usize) -> usize {
    while parent[node] != node {
        parent[node] = parent[parent[node]];
        node = parent[node];
    }
    node
}

/// Connector lines of a diagram (an organisation chart, a tree, a flow) that
/// end in the air a short way from a box they do not touch. Lines and boxes
/// that touch form one piece of the diagram; a loose end next to another
/// piece leaves the diagram in two. Only lines drawn like the connectors of
/// the page are judged: a rule, an axis or a line that touches nothing is no
/// connector.
fn broken_connectors(
    world: &LabWorld,
    geometry: &Geometry,
    body: f64,
) -> Vec<Diagnostic> {
    let reach = MAX_CONNECTOR_GAP_EM * body;
    let mut out = Vec::new();
    for page in 0..geometry.pages.len() {
        let lines = strokes(geometry, page);
        let marks: Vec<BBox> = geometry
            .items
            .iter()
            .filter(|i| i.page == page)
            .filter(|i| match i.kind {
                ItemKind::Shape => line_thickness(&i.bbox).is_none(),
                ItemKind::Image => true,
                _ => false,
            })
            .map(|i| i.bbox)
            .filter(|b| b.width() >= MIN_MARK_PT && b.height() >= MIN_MARK_PT)
            .collect();
        if lines.is_empty() || marks.is_empty() {
            continue;
        }
        let rects: Vec<BBox> = lines.iter().map(Stroke::bbox).collect();
        let texts: Vec<BBox> = geometry
            .items
            .iter()
            .filter(|i| {
                i.page == page
                    && i.kind == ItemKind::Text
                    && i.text.as_deref().is_some_and(|t| !t.trim().is_empty())
            })
            .map(|i| i.bbox)
            .collect();

        // The pieces of the page: lines and boxes that touch. A box that
        // holds a line is its background, not its neighbour.
        let count = lines.len();
        let mut parent: Vec<usize> = (0..count + marks.len()).collect();
        for a in 0..count {
            for b in a + 1..count {
                if meet(&rects[a], &rects[b]) {
                    let (x, y) = (root(&mut parent, a), root(&mut parent, b));
                    parent[x] = y;
                }
            }
            for (b, mark) in marks.iter().enumerate() {
                if meet(&rects[a], mark) && !encloses(mark, &rects[a]) {
                    let (x, y) = (root(&mut parent, a), root(&mut parent, count + b));
                    parent[x] = y;
                }
            }
        }
        // A fill and its outline are one box.
        for a in 0..marks.len() {
            for b in a + 1..marks.len() {
                let (m, n) = (&marks[a], &marks[b]);
                if (m.x0 - n.x0).abs() <= 1.5
                    && (m.y0 - n.y0).abs() <= 1.5
                    && (m.x1 - n.x1).abs() <= 1.5
                    && (m.y1 - n.y1).abs() <= 1.5
                {
                    let (x, y) =
                        (root(&mut parent, count + a), root(&mut parent, count + b));
                    parent[x] = y;
                }
            }
        }
        let piece: Vec<usize> =
            (0..parent.len()).map(|node| root(&mut parent, node)).collect();

        // What each end of a line joins: a box, another line, a label, or
        // nothing.
        let joins = |index: usize, last: bool| -> (bool, bool) {
            let (tip, other) = (lines[index].tip(last), lines[index].tip(!last));
            let on_mark = marks.iter().any(|mark| {
                holds(&grown(mark, TOUCH_PT), tip)
                    && !holds(&grown(mark, -TOUCH_PT), other)
            });
            let joined = on_mark
                || rects
                    .iter()
                    .enumerate()
                    .any(|(k, rect)| k != index && holds(&grown(rect, TOUCH_PT), tip))
                || texts.iter().any(|text| distance(text, tip) <= LABEL_REACH_PT);
            (on_mark, joined)
        };
        let ends: Vec<[(bool, bool); 2]> = (0..count)
            .map(|index| [joins(index, false), joins(index, true)])
            .collect();
        let connectors: Vec<f64> = (0..count)
            .filter(|&k| {
                ends[k].iter().all(|end| end.1) && ends[k].iter().any(|end| end.0)
            })
            .map(|k| lines[k].thickness)
            .collect();

        for (index, line) in lines.iter().enumerate() {
            let alike = connectors
                .iter()
                .filter(|t| (*t - line.thickness).abs() <= SAME_STROKE_PT)
                .count();
            let alone = !(0..piece.len()).any(|k| k != index && piece[k] == piece[index]);
            if alike < MIN_CONNECTORS || alone {
                continue;
            }
            let mut miss: Option<(f64, BBox)> = None;
            for last in [false, true] {
                if ends[index][last as usize].1 {
                    continue;
                }
                let tip = line.tip(last);
                let sign = if last { 1.0 } else { -1.0 };
                for (k, mark) in marks.iter().enumerate() {
                    if piece[count + k] == piece[index] || encloses(mark, &rects[index]) {
                        continue;
                    }
                    let (x, y) = nearest(mark, tip);
                    let along = sign * if line.vertical { y - tip.1 } else { x - tip.0 };
                    let gap = distance(mark, tip);
                    if along >= -BEHIND_PT
                        && gap <= reach
                        && miss.is_none_or(|(least, _)| gap < least)
                    {
                        miss = Some((gap, *mark));
                    }
                }
            }
            let Some((gap, mark)) = miss else { continue };
            let item = &geometry.items[line.item];
            let mut diagnostic = Diagnostic::new(
                BROKEN_CONNECTOR,
                Severity::Warning,
                format!(
                    "a connector line on page {} ends in the air, {}pt from a box it does \
                     not touch, so the diagram falls apart there",
                    page + 1,
                    round1(gap)
                ),
            );
            diagnostic.loc =
                diag::resolve_span(world, geometry.span_for(item.span, item.elem));
            diagnostic.page = Some(page + 1);
            diagnostic.bbox = Some(rects[index].union(&mark).rounded());
            diagnostic.measured = Some(round1(gap));
            diagnostic.limit = Some(TOUCH_PT);
            diagnostic.unit = Some("pt");
            diagnostic.hints.push(
                "make both ends of every connector touch what they join: draw each drop in \
                 the same grid column as the box it joins and as tall as the gap, run a \
                 bar from the first drop to the last, and start a trunk at the edge of its \
                 box"
                    .into(),
            );
            out.push(diagnostic);
        }
    }
    out
}

