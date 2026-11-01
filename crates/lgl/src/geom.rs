use std::collections::HashMap;

use serde::Serialize;
use typst::introspection::{Location, Tag};
use typst::layout::{Frame, FrameItem, Point, Rect, Transform};
use typst::syntax::Span;
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
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip)]
    pub span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub struct Element {
    pub name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
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
pub struct PageInfo {
    pub index: usize,
    pub number: u64,
    pub width: f64,
    pub height: f64,
    pub bleed: [f64; 4],
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Geometry {
    pub pages: Vec<PageInfo>,
    pub elements: Vec<Element>,
    pub items: Vec<Item>,
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
        walker.frame(&page.frame, Transform::identity(), None);
    }
    walker.finish()
}

#[derive(Default)]
struct Walker {
    geometry: Geometry,
    page: usize,
    open: Vec<usize>,
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
                    self.frame(&group.frame, group_ts, group_clip);
                }
                FrameItem::Text(text) => {
                    let span = text
                        .glyphs
                        .iter()
                        .map(|glyph| glyph.span.0)
                        .find(|span| !span.is_detached())
                        .unwrap_or_else(Span::detached);
                    self.push(
                        ItemKind::Text,
                        text.bbox(),
                        item_ts,
                        clip,
                        span,
                        Some(text.text.to_string()),
                        Some(text.size.to_pt()),
                    );
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
                FrameItem::Image(_, size, span) => {
                    self.push(
                        ItemKind::Image,
                        Rect::from_pos_size(Point::zero(), *size),
                        item_ts,
                        clip,
                        *span,
                        None,
                        None,
                    );
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
    ) {
        let Some(bbox) = BBox::transformed(local, ts) else { return };
        self.geometry.items.push(Item {
            kind,
            page: self.page,
            bbox,
            clip,
            elem: self.open.last().copied(),
            text,
            font_size,
            span,
        });
    }

    fn finish(mut self) -> Geometry {
        let items = std::mem::take(&mut self.geometry.items);
        for item in &items {
            let visible = match item.clip {
                Some(clip) => match item.bbox.intersect(&clip) {
                    Some(visible) => visible,
                    None => continue,
                },
                None => item.bbox,
            };
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
