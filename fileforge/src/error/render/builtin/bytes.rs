use crate::error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable};

/// Bytes as hex (`FE FF`), followed by the text they spell when every byte is printable ASCII
/// (`42 59 ("BY")`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Bytes<const N: usize>(pub [u8; N]);

impl<'t, const N: usize> Renderable<'t> for Bytes<N> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    for (index, byte) in self.0.iter().enumerate() {
      if index != 0 {
        canvas.set_str(" ");
      }

      canvas.write(&FormattedUnsigned::new(*byte as u128).base(16).uppercase().padding(2))?;
    }

    if N != 0 && self.0.iter().all(u8::is_ascii_graphic) {
      canvas.set_str(" (\"");
      canvas.set_str(core::str::from_utf8(&self.0).map_err(|_| ())?);
      canvas.set_str("\")");
    }

    Ok(())
  }
}
