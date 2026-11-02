//! Invisible layout facts for external quality checks.

use typst_syntax::Span;
use typst_utils::Scalar;

use crate::layout::{Abs, Size};

/// An invisible record of a layout decision.
///
/// Probes are only emitted when [`Library::probes`](crate::Library::probes) is
/// enabled. Exporters ignore them. They exist so that tools can verify the
/// geometry of a finished document without rendering it.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Probe {
    /// Starts a container whose box spans `size` from the probe's position.
    /// Everything up to the matching [`Probe::ContainerEnd`] belongs to it.
    ContainerStart(ContainerProbe),
    /// Ends the most recently started container.
    ContainerEnd,
    /// Starts an area available to flow content, spanning `size` from the
    /// probe's position. Everything up to the matching [`Probe::RegionEnd`]
    /// is laid out within it.
    RegionStart(RegionProbe),
    /// Ends the most recently started region.
    RegionEnd,
    /// Facts about a single committed line of a paragraph. The probe sits at
    /// the line's top-left corner.
    Line(LineProbe),
}

/// A container whose content is expected to stay within its bounds.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct ContainerProbe {
    /// What kind of container this is.
    pub kind: ContainerKind,
    /// The size of the container.
    pub size: Size,
    /// Whether the container clips its content.
    pub clip: bool,
    /// Whether the container has an explicit, non-automatic width or height.
    pub sized: bool,
    /// Whether the container has a visible fill or stroke.
    pub visible: bool,
    /// The span of the element that produced the container.
    pub span: Span,
}

/// The kind of a [`ContainerProbe`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ContainerKind {
    /// An inline box.
    Box,
    /// A block or one fragment of a breakable block.
    Block,
    /// A grid or table cell.
    Cell,
}

/// An area available to flow content.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct RegionProbe {
    /// What kind of region this is.
    pub kind: RegionKind,
    /// The size of the region.
    pub size: Size,
    /// Whether this region ends a run of pages, i.e. whether it was ended by
    /// a page break or the end of the document rather than by running out of
    /// space.
    pub last_in_run: bool,
    /// The pitch of the baseline grid of this region, if any.
    pub grid: Option<Abs>,
}

/// The kind of a [`RegionProbe`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum RegionKind {
    /// The page area inside the margins.
    PageBody,
    /// The area of the page header.
    Header,
    /// The area of the page footer.
    Footer,
    /// The page background, including bleed.
    Background,
    /// The page foreground, including bleed.
    Foreground,
    /// One column of a multi-column flow.
    Column,
}

/// Facts about a committed paragraph line.
#[derive(Debug, Clone, PartialEq, Hash)]
pub struct LineProbe {
    /// The width available to the line.
    pub available: Abs,
    /// The height of the line.
    pub height: Abs,
    /// The distance from the top of the line to its baseline.
    pub baseline: Abs,
    /// The natural width of the line's content.
    pub natural: Abs,
    /// The width by which the content exceeds the available width after all
    /// shrinking. Zero if the line fits.
    pub overflow: Abs,
    /// The first-step justification ratio in `[-1, 1]`. Negative values mean
    /// the line was shrunk, positive values that it was stretched.
    pub ratio: Scalar,
    /// Extra space added after each justifiable character because the
    /// stretchability was exhausted.
    pub extra: Abs,
    /// Whether the line is justified.
    pub justify: bool,
    /// Whether the line ends in a hyphen inserted by hyphenation.
    pub hyphenated: bool,
    /// The span of the line's first text, if any.
    pub span: Span,
}
