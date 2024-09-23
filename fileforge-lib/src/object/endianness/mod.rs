use error::EndiannessMarkerError;

use crate::{diagnostic::node::tagged_reference::TaggedDiagnosticReference, error::render::{buffer::canvas::RenderBufferCanvas, builtin::number::formatted_unsigned::FormattedUnsigned, r#trait::renderable::Renderable}, provider::r#trait::Provider, reader::{self, error::parse::ParseError, r#trait::readable::FixedSizeReadable, Reader}};

use super::magic::Magic;

pub mod error;

#[derive(PartialEq, Eq, Clone)]
pub struct EndiannessMarker<const SIZE: usize> {
  bytes: [u8; SIZE],
  endianness: reader::endianness::Endianness,
}

impl<const SIZE: usize> EndiannessMarker<SIZE> {
  pub fn big(bytes: [u8; SIZE]) -> Self {
    Self { bytes, endianness: reader::endianness::Endianness::Big }
  }

  pub fn little(bytes: [u8; SIZE]) -> Self {
    Self { bytes, endianness: reader::endianness::Endianness::Little }
  }

  pub fn endianness(&self) -> reader::endianness::Endianness {
    self.endianness
  }

  pub fn inverse_clone(&self) -> EndiannessMarker<SIZE> {
    let mut bytes_clone = self.bytes;
    bytes_clone.reverse();

    EndiannessMarker { bytes: bytes_clone, endianness: self.endianness.inverse() }
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, const SIZE: usize> FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, SIZE> for EndiannessMarker<SIZE> {
  type Argument = EndiannessMarker<SIZE>;
  type Error = EndiannessMarkerError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, SIZE>;

  fn read<RP: Provider>(reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>, expected: Self::Argument) -> Result<Self, ParseError<'pool, Self::Error, RP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let mut bytes: [u8; SIZE] = reader.get("Endianness")?;

    if bytes == expected.bytes {
      return Ok(expected)
    }

    bytes.reverse();

    if bytes == expected.bytes {
      return Ok(EndiannessMarker::little(bytes))
    }

    Err(ParseError::domain_err(EndiannessMarkerError {
      expected,
      actual: TaggedDiagnosticReference::tag(Magic::from_bytes(bytes), reader.diagnostic_reference())
    }))
  }
}

impl<'t, const SIZE: usize> Renderable<'t> for EndiannessMarker<SIZE> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_str("EndiannessMarker::<");
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

    canvas.set_str(", ");

    if self.endianness == reader::endianness::Endianness::Big {
      canvas.set_str("Endianness::Big")
    } else {
      canvas.set_str("Endianness::Little")
    }

    canvas.set_char(")");

    Ok(())
  }
}