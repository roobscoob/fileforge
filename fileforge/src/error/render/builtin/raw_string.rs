use crate::error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable};

pub struct RawString<'a>(pub &'a [u8]);

impl<'a, 't> Renderable<'t> for RawString<'a> {
  fn render_into<'buffer_reference, 'buffer_contents>(&self, canvas: &mut RenderBufferCanvas<'buffer_reference, 'buffer_contents, 't>) -> Result<(), ()> {
    Ok(match core::str::from_utf8(&self.0) {
      Ok(string) => {
        canvas.set_str("b'");
        canvas.set_str(string);
        canvas.set_str("'");
      }

      Err(..) => {
        canvas.set_char("0x");

        for (_, byte) in self.0.iter().enumerate() {
          canvas.write(&FormattedUnsigned::new(*byte as u128).padding(2).base(16).uppercase())?;
        }
      }
    })
  }
}
