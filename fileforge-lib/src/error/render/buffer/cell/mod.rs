use crate::error::render::grapheme::Grapheme;

use self::tag::CellTag;

pub mod tag;

#[derive(Default, Clone, Copy)]
pub struct RenderBufferCell<'tag> {
  contents: Grapheme,
  tag: Option<&'tag dyn CellTag>,
}

impl<'tag> RenderBufferCell<'tag> {
  pub fn new(contents: Grapheme) -> Self {
    Self {
      contents,
      tag: None,
    }
  }

  pub fn clear(&mut self) {
    self.contents = Grapheme::default();
    self.tag = None;
  }

  pub fn from_str(grapheme: &str) -> Self { Self::new(Grapheme::from_str(grapheme)) }

  pub fn with_tag(self, tag: &'tag dyn CellTag) -> Self {
    Self {
      contents: self.contents,
      tag: Some(tag),
    }
  }

  pub fn contents(&self) -> &Grapheme { &self.contents }

  pub fn tag(&self) -> Option<&'tag dyn CellTag> { self.tag }

  pub fn width(&self) -> usize { self.contents.width() }
}
