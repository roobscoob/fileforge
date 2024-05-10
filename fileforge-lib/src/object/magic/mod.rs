use crate::{diagnostic::node::tagged_reference::TaggedDiagnosticReference, error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable}, provider::r#trait::Provider, reader::{error::ParseError, r#trait::readable::FixedSizeReadable, Reader}};

use self::error::MagicError;

pub mod error;

#[derive(PartialEq, Eq)]
pub struct Magic<const SIZE: usize> {
  bytes: [u8; SIZE],
}

impl<const SIZE: usize> Magic<SIZE> {
  pub fn from_bytes(bytes: [u8; SIZE]) -> Magic<SIZE> {
    Self { bytes }
  }
}

impl<'pool_lifetime, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const SIZE: usize> FixedSizeReadable<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, SIZE> for Magic<SIZE> {
  type Argument = Magic<SIZE>;
  type Error = MagicError<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, SIZE>;

  fn read<'rl, RP: Provider>(reader: &mut Reader<'pool_lifetime, 'rl, DIAGNOSTIC_NODE_NAME_SIZE, RP>, expected: Self::Argument) -> Result<Self, ParseError<'pool_lifetime, Self::Error, RP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let actual = Self::from_bytes(reader.get("Bytes")?);

    if actual != expected {
      return Err(ParseError::domain_err(MagicError {
        expected,
        actual: TaggedDiagnosticReference::tag(actual, reader.diagnostic_reference())
      }));
    }

    Ok(actual)
  }
}

impl<'t, const SIZE: usize> Renderable<'t> for Magic<SIZE> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_str("Magic::<");
    canvas.write(&FormattedUnsigned::new(SIZE as u64))?;
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
          canvas.write(&FormattedUnsigned::new(*byte as u64).with_padding(2).with_base(16).with_uppercase())?;

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