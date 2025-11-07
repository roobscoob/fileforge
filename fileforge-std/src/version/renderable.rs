use fileforge::error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable};

use super::Version;

impl<'t, const SEGMENTS: usize> Renderable<'t> for Version<SEGMENTS> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_str("v");

    for (index, segment) in self.versions.iter().enumerate() {
      let number = FormattedUnsigned::new(Into::<u128>::into(*segment));

      canvas.write(&number)?;

      if index != self.versions.len() - 1 {
        canvas.set_str(".");
      }
    }

    Ok(())
  }
}
