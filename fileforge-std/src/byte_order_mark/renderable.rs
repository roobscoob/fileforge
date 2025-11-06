use fileforge::error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable};

use super::ByteOrderMark;

impl<'t> Renderable<'t> for ByteOrderMark {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_str("ByteOrderMark(BigEndian: ");
    let be = self.be().bytes();
    match core::str::from_utf8(&be) {
      Ok(string) => {
        canvas.set_str("b'");
        canvas.set_str(string);
        canvas.set_str("'");
      }

      Err(..) => {
        canvas.set_char("0x");

        for byte in be.iter() {
          canvas.write(&FormattedUnsigned::new(*byte as u128).padding(2).base(16).uppercase())?;
        }
      }
    }
    canvas.set_char(", LittleEndian: ");
    let le = self.le().bytes();
    match core::str::from_utf8(&le) {
      Ok(string) => {
        canvas.set_str("b'");
        canvas.set_str(string);
        canvas.set_str("'");
      }

      Err(..) => {
        canvas.set_char("0x");

        for byte in le.iter() {
          canvas.write(&FormattedUnsigned::new(*byte as u128).padding(2).base(16).uppercase())?;
        }
      }
    }
    canvas.set_char(")");

    Ok(())
  }
}
