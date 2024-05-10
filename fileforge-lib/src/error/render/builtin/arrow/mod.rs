use crate::error::render::{buffer::canvas::RenderBufferCanvas, r#trait::renderable::Renderable};

use self::{primary::PrimaryArrow, secondary::SecondaryArrow};

use super::transformation::Transformation;

pub mod primary;
pub mod secondary;

pub enum EitherArrow {
  Primary(PrimaryArrow),
  Secondary(SecondaryArrow),
}

impl EitherArrow {
  pub fn with_transformation(self, transformation: Option<Transformation>) -> Self {
    match self {
      Self::Primary(pa) => EitherArrow::Primary(PrimaryArrow {
        transformation,
        ..pa
      }),

      Self::Secondary(sa) => EitherArrow::Secondary(SecondaryArrow {
        transformation,
        ..sa
      })
    }
  }
}

impl<'t> Renderable<'t> for EitherArrow {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    match self {
      EitherArrow::Primary(p) => p.render_into(canvas),
      EitherArrow::Secondary(s) => s.render_into(canvas),
    }
  }
}