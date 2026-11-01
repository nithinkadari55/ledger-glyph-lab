use std::collections::HashMap;

use serde::Serialize;
use typst::introspection::{Location, Tag};
use typst::layout::{
    ContainerKind, Frame, FrameItem, Point, Probe, Rect, RegionKind, Size, Transform,
};
use typst::model::RefElem;
use typst::syntax::Span;
use typst::visualize::ImageKind;
use typst_layout::PagedDocument;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BBox {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl BBox {
    fn transformed(rect: Rect, ts: Transform) -> Option<Self> {
        let corners = [
            rect.min,
            Point::new(rect.max.x, rect.min.y),
            rect.max,
            Point::new(rect.min.x, rect.max.y),
        ];
        let mut out = Self {
            x0: f64::INFINITY,
            y0: f64::INFINITY,
            x1: f64::NEG_INFINITY,
            y1: f64::NEG_INFINITY,
        };
        for corner in corners {
            let p = corner.transform(ts);
            out.x0 = out.x0.min(p.x.to_pt());
            out.y0 = out.y0.min(p.y.to_pt());
            out.x1 = out.x1.max(p.x.to_pt());
            out.y1 = out.y1.max(p.y.to_pt());
        }
        out.is_finite().then_some(out)
    }

    fn sized(size: Size, ts: Transform) -> Option<Self> {
        Self::transformed(Rect::from_pos_size(Point::zero(), size), ts)
    }

    pub fn is_finite(&self) -> bool {
        self.x0.is_finite()
            && self.y0.is_finite()
            && self.x1.is_finite()
            && self.y1.is_finite()
    }

    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            x0: self.x0.min(other.x0),
            y0: self.y0.min(other.y0),
            x1: self.x1.max(other.x1),
            y1: self.y1.max(other.y1),
        }
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let out = Self {
            x0: self.x0.max(other.x0),
            y0: self.y0.max(other.y0),
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
        };
        (out.x0 <= out.x1 && out.y0 <= out.y1).then_some(out)
    }

    /// How far `self` extends beyond `bounds`, as the largest excess on any
    /// side together with the side's name.
    pub fn excess(&self, bounds: &Self) -> (f64, &'static str) {
        [
            (bounds.x0 - self.x0, "left"),
            (bounds.y0 - self.y0, "top"),
            (self.x1 - bounds.x1, "right"),
            (self.y1 - bounds.y1, "bottom"),
        ]
        .into_iter()
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .unwrap()
    }

    pub fn rounded(&self) -> Self {
        Self {
            x0: round1(self.x0),
            y0: round1(self.y0),
            x1: round1(self.x1),
            y1: round1(self.y1),
        }
    }
}

pub fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Text,
    Shape,
    Image,
    Link,
}

#[derive(Debug, Clone, Serialize)]
pub struct Item {
    pub kind: ItemKind,
    pub page: usize,
    pub bbox: BBox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clip: Option<BBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elem: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pixels: Option<[u32; 2]>,
    #[serde(skip)]
    pub span: Span,
}

impl Item {
    /// The part of the item that is not clipped away.
    pub fn visible(&self) -> Option<BBox> {
        match self.clip {
            Some(clip) => self.bbox.intersect(&clip),
            None => Some(self.bbox),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Element {
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<usize>,
    pub boxes: Vec<PageBox>,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PageBox {
    pub page: usize,
    pub bbox: BBox,
}

#[derive(Debug, Clone, Serialize)]
pub struct Container {
    pub kind: &'static str,
    pub page: usize,
    pub bbox: BBox,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<usize>,
    pub clip: bool,
    pub sized: bool,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elem: Option<usize>,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Region {
    pub kind: &'static str,
    pub page: usize,
    pub bbox: BBox,
    pub last_in_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Line {
    pub page: usize,
    pub bbox: BBox,
    pub available: f64,
    pub natural: f64,
    pub overflow: f64,
    pub ratio: f64,
    pub extra: f64,
    pub justify: bool,
    pub hyphenated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elem: Option<usize>,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct PageInfo {
    pub index: usize,
    pub number: u64,
    pub width: f64,
    pub height: f64,
    pub bleed: [f64; 4],
}

impl PageInfo {
    /// The page area including bleed.
    pub fn bounds(&self) -> BBox {
        let [left, top, right, bottom] = self.bleed;
        BBox {
            x0: -left,
            y0: -top,
            x1: self.width + right,
            y1: self.height + bottom,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Geometry {
    pub pages: Vec<PageInfo>,
    pub elements: Vec<Element>,
    pub containers: Vec<Container>,
    pub regions: Vec<Region>,
    pub lines: Vec<Line>,
    pub items: Vec<Item>,
}

impl Geometry {
    /// Whether a region is a page body, as opposed to a header or footer.
    pub fn is_body(&self, region: Option<usize>) -> bool {
        region.is_some_and(|r| self.regions[r].kind == "page-body")
    }
}

pub fn extract(document: &PagedDocument) -> Geometry {
    let mut walker = Walker::default();
    for (index, page) in document.pages().iter().enumerate() {
        let size = page.frame.size();
        walker.geometry.pages.push(PageInfo {
            index,
            number: page.number,
            width: size.x.to_pt(),
            height: size.y.to_pt(),
            bleed: [
                page.bleed.left.to_pt(),
                page.bleed.top.to_pt(),
                page.bleed.right.to_pt(),
                page.bleed.bottom.to_pt(),
            ],
        });
        walker.page = index;
        walker.containers.clear();
        walker.regions.clear();
        walker.frame(&page.frame, Transform::identity(), None);
    }
    walker.finish()
}

#[derive(Default)]
struct Walker {
    geometry: Geometry,
    page: usize,
    open: Vec<usize>,
    containers: Vec<usize>,
    regions: Vec<usize>,
    by_location: HashMap<Location, usize>,
}

impl Walker {
    fn frame(&mut self, frame: &Frame, ts: Transform, clip: Option<BBox>) {
        for (pos, item) in frame.items() {
            let item_ts = ts.pre_concat(Transform::translate(pos.x, pos.y));
            match item {
                FrameItem::Group(group) => {
                    let group_ts = item_ts.pre_concat(group.transform);
                    let group_clip = match &group.clip {
                        Some(curve) => {
                            let local = curve.bbox(None);
                            match (BBox::transformed(local, group_ts), clip) {
                                (Some(own), Some(outer)) => {
                                    Some(own.intersect(&outer).unwrap_or(BBox {
                                        x0: own.x0,
                                        y0: own.y0,
                                        x1: own.x0,
                                        y1: own.y0,
                                    }))
                                }
                                (Some(own), None) => Some(own),
                                (None, outer) => outer,
                            }
                        }
                        None => clip,
                    };
                    let depth = (self.containers.len(), self.regions.len());
                    self.frame(&group.frame, group_ts, group_clip);
                    self.containers.truncate(depth.0);
                    self.regions.truncate(depth.1);
                }
                FrameItem::Text(text) => {
                    let span = text
                        .glyphs
                        .iter()
                        .map(|glyph| glyph.span.0)
                        .find(|span| !span.is_detached())
                        .unwrap_or_else(Span::detached);
                    let missing: String = text
                        .glyphs
                        .iter()
                        .filter(|glyph| glyph.id == 0)
                        .filter_map(|glyph| text.text.get(glyph.range()))
                        .collect();
                    if let Some(item) = self.push(
                        ItemKind::Text,
                        text.bbox(),
                        item_ts,
                        clip,
                        span,
                        Some(text.text.to_string()),
                        Some(text.size.to_pt()),
                    ) && !missing.is_empty()
                    {
                        item.missing = Some(missing);
                    }
                }
                FrameItem::Shape(shape, span) => {
                    self.push(
                        ItemKind::Shape,
                        shape.bbox(true),
                        item_ts,
                        clip,
                        *span,
                        None,
                        None,
                    );
                }
                FrameItem::Image(image, size, span) => {
                    if let Some(item) = self.push(
                        ItemKind::Image,
                        Rect::from_pos_size(Point::zero(), *size),
                        item_ts,
                        clip,
                        *span,
                        None,
                        None,
                    ) && let ImageKind::Raster(raster) = image.kind()
                    {
                        item.pixels = Some([raster.width(), raster.height()]);
                    }
                }
                FrameItem::Link(_, size) => {
                    self.push(
                        ItemKind::Link,
                        Rect::from_pos_size(Point::zero(), *size),
                        item_ts,
                        clip,
                        Span::detached(),
                        None,
                        None,
                    );
                }
                FrameItem::Tag(tag) => self.tag(tag),
                FrameItem::Probe(probe) => self.probe(probe, item_ts),
            }
        }
    }

    fn probe(&mut self, probe: &Probe, ts: Transform) {
        match probe {
            Probe::ContainerStart(container) => {
                let Some(bbox) = BBox::sized(container.size, ts) else { return };
                let index = self.geometry.containers.len();
                self.geometry.containers.push(Container {
                    kind: match container.kind {
                        ContainerKind::Box => "box",
                        ContainerKind::Block => "block",
                        ContainerKind::Cell => "cell",
                    },
                    page: self.page,
                    bbox,
                    region: self.regions.last().copied(),
                    clip: container.clip,
                    sized: container.sized,
                    visible: container.visible,
                    parent: self.containers.last().copied(),
                    elem: self.open.last().copied(),
                    span: container.span,
                });
                self.containers.push(index);
            }
            Probe::ContainerEnd => {
                self.containers.pop();
            }
            Probe::RegionStart(region) => {
                let Some(bbox) = BBox::sized(region.size, ts) else { return };
                let index = self.geometry.regions.len();
                self.geometry.regions.push(Region {
                    kind: match region.kind {
                        RegionKind::PageBody => "page-body",
                        RegionKind::Header => "header",
                        RegionKind::Footer => "footer",
                    },
                    page: self.page,
                    bbox,
                    last_in_run: region.last_in_run,
                });
                self.regions.push(index);
            }
            Probe::RegionEnd => {
                self.regions.pop();
            }
            Probe::Line(line) => {
                let size = Size::new(line.available, line.height);
                let Some(bbox) = BBox::sized(size, ts) else { return };
                self.geometry.lines.push(Line {
                    page: self.page,
                    bbox,
                    available: line.available.to_pt(),
                    natural: line.natural.to_pt(),
                    overflow: line.overflow.to_pt(),
                    ratio: line.ratio.get(),
                    extra: line.extra.to_pt(),
                    justify: line.justify,
                    hyphenated: line.hyphenated,
                    container: self.containers.last().copied(),
                    elem: self.open.last().copied(),
                    span: line.span,
                });
            }
        }
    }

    fn tag(&mut self, tag: &Tag) {
        match tag {
            Tag::Start(content, _) => {
                let Some(location) = content.location() else { return };
                let index = self.geometry.elements.len();
                self.geometry.elements.push(Element {
                    name: content.elem().name(),
                    label: content.label().map(|label| label.resolve().to_string()),
                    target: content
                        .to_packed::<RefElem>()
                        .map(|r| r.target.resolve().to_string()),
                    parent: self.open.last().copied(),
                    boxes: Vec::new(),
                    span: content.span(),
                });
                self.by_location.insert(location, index);
                self.open.push(index);
            }
            Tag::End(location, ..) => {
                if let Some(&index) = self.by_location.get(location)
                    && let Some(pos) = self.open.iter().rposition(|&open| open == index)
                {
                    self.open.remove(pos);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        kind: ItemKind,
        local: Rect,
        ts: Transform,
        clip: Option<BBox>,
        span: Span,
        text: Option<String>,
        font_size: Option<f64>,
    ) -> Option<&mut Item> {
        let bbox = BBox::transformed(local, ts)?;
        self.geometry.items.push(Item {
            kind,
            page: self.page,
            bbox,
            clip,
            elem: self.open.last().copied(),
            container: self.containers.last().copied(),
            region: self.regions.last().copied(),
            text,
            font_size,
            missing: None,
            pixels: None,
            span,
        });
        self.geometry.items.last_mut()
    }

    fn finish(mut self) -> Geometry {
        let items = std::mem::take(&mut self.geometry.items);
        for item in &items {
            let Some(visible) = item.visible() else { continue };
            let mut cursor = item.elem;
            while let Some(index) = cursor {
                let element = &mut self.geometry.elements[index];
                match element.boxes.iter_mut().find(|b| b.page == item.page) {
                    Some(existing) => existing.bbox = existing.bbox.union(&visible),
                    None => {
                        element.boxes.push(PageBox { page: item.page, bbox: visible })
                    }
                }
                cursor = element.parent;
            }
        }
        self.geometry.items = items;
        self.geometry
    }
}
