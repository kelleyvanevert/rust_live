use emath::Rect;
use epaint::{ahash::HashMap, ClippedShape, Shape};
use itertools::Itertools;

/// A unique identifier of a specific [`Shape`] in a [`PaintList`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ShapeIdx(usize);

/// A list of [`Shape`]s paired with a clip rectangle.
#[derive(Clone, Default)]
pub struct Layer(Vec<ClippedShape>);

impl Layer {
    /// Returns the index of the new [`Shape`] that can be used with `PaintList::set`.
    #[inline(always)]
    pub fn add(&mut self, clip_rect: Rect, shape: Shape) -> ShapeIdx {
        let idx = ShapeIdx(self.0.len());
        self.0.push(ClippedShape(clip_rect, shape));
        idx
    }
}

#[derive(Clone, Default)]
pub(crate) struct GraphicLayers(HashMap<usize, Layer>);

impl GraphicLayers {
    pub fn layer(&mut self, index: usize) -> &mut Layer {
        self.0.entry(index).or_default()
    }

    pub fn drain(&mut self) -> impl ExactSizeIterator<Item = ClippedShape> {
        let mut all_shapes: Vec<_> = Default::default();

        for (_index, layer) in self.0.iter_mut().sorted_by_key(|p| p.0) {
            all_shapes.append(&mut layer.0);
        }

        all_shapes.into_iter()
    }
}
