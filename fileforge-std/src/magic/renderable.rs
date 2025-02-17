use fileforge_lib::error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable};

use super::Magic;

impl<'t, const SIZE: usize> Renderable<'t> for Magic<SIZE> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_str("Magic::<");
    canvas.write(&FormattedUnsigned::new(SIZE as u128))?;
    canvas.set_str(">(");
    match core::str::from_utf8(&self.bytes) {
      Ok(string) => {
        canvas.set_str("b'");
        canvas.set_str(string);
        canvas.set_str("'");
      }

      Err(..) => {
        canvas.set_char("0x");

        for (index, byte) in self.bytes.iter().enumerate() {
          canvas.write(&FormattedUnsigned::new(*byte as u128).padding(2).base(16).uppercase())?;

          if index != SIZE - 1 {
            canvas.set_char(" ");
          }
        }
      }
    }
    canvas.set_char(")");

    Ok(())
  }
}
