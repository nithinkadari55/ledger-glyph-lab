//! Emission of invisible layout probes.

use typst_library::engine::Engine;
use typst_library::layout::{
    ContainerKind, ContainerProbe, Frame, FrameItem, LineProbe, Point, Probe,
};
use typst_syntax::Span;

/// Marks the extent of a container in its frame, if probes are enabled.
///
/// Empty frames are left untouched so that probes never influence layout
/// decisions that depend on whether a frame is empty.
pub fn container(
    engine: &Engine,
    frame: &mut Frame,
    kind: ContainerKind,
    clip: bool,
    span: Span,
) {
    if !engine.library.probes || frame.is_empty() {
        return;
    }

    let probe = ContainerProbe { kind, size: frame.size(), clip, span };
    frame.prepend(Point::zero(), FrameItem::Probe(Probe::ContainerStart(probe)));
    frame.push(Point::zero(), FrameItem::Probe(Probe::ContainerEnd));
}

/// Attaches facts about a committed paragraph line to its frame, if probes are
/// enabled.
///
/// The probe's span is filled in from the line's first text.
pub fn line(engine: &Engine, frame: &mut Frame, mut probe: LineProbe) {
    if !engine.library.probes || frame.is_empty() {
        return;
    }

    probe.span = frame
        .items()
        .find_map(|(_, item)| match item {
            FrameItem::Text(text) => text
                .glyphs
                .iter()
                .map(|glyph| glyph.span.0)
                .find(|span| !span.is_detached()),
            _ => None,
        })
        .unwrap_or_else(Span::detached);

    frame.prepend(Point::zero(), FrameItem::Probe(Probe::Line(probe)));
}
