use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  diagnostic::value::{DiagnosticSaturation, DiagnosticValue},
  error::{
    ext::annotations::annotated::Annotated,
    render::{
      buffer::canvas::RenderBufferCanvas,
      builtin::{number::formatted_unsigned::FormattedUnsigned, raw_string::RawString},
      r#trait::renderable::Renderable,
    },
  },
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_macros::FileforgeError;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Magic<const SIZE: usize> {
  bytes: [u8; SIZE],
}

/**
 * Case 1. "Primary Magic" (Header of a file) is wrong:
 * - Data is for another file type
 *
 * Case 2. All Magics:
 * - Data is corrupted
 * - Your provider could have fucked up
 *   - e.g. Failed to properly decrypt file
 *   - e.g. Failed to properly decompress file
 *   - e.g. Invalid pointer
 */

#[derive(FileforgeError)]
pub enum MagicError<'pool, const MAGIC_SIZE: usize, U: UserReadError> {
  Failed(#[from] Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),

  #[report(&"Invalid Magic")]
  #[flag("TODO: Write a flag line")]
  #[info("TODO: Write an info line")]
  Invalid {
    #[error("Found {actual.value()}, expected {expected}")]
    actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>>,
    expected: Magic<MAGIC_SIZE>,
  },
}

impl<const SIZE: usize> Magic<SIZE> {
  pub const fn from_bytes(bytes: [u8; SIZE]) -> Magic<SIZE> {
    Self { bytes }
  }
  pub const fn from_byte_ref(bytes: &[u8; SIZE]) -> Magic<SIZE> {
    Self { bytes: *bytes }
  }
}

impl<'pool, const SIZE: usize, S: ReadableStream<Type = u8>> Readable<'pool, S> for Magic<SIZE> {
  type Error = MagicError<'pool, SIZE, S::ReadError>;
  type Argument = Magic<SIZE>;

  async fn read(reader: &mut BinaryReader<'pool, S>, expected: Self::Argument) -> Result<Self, Self::Error> {
    let actual = Self::from_bytes(reader.get::<[u8; SIZE]>().await?);

    if expected != actual {
      return Err(MagicError::Invalid {
        actual: reader.create_physical_diagnostic(-(SIZE as i128), Some(SIZE as u64), "Magic").saturate(actual),
        expected,
      });
    }

    Ok(actual)
  }
}

impl<'t, const SIZE: usize> Renderable<'t> for Magic<SIZE> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    canvas.set_str("Magic::<");
    canvas.write(&FormattedUnsigned::from(SIZE))?;
    canvas.set_str(">(");
    canvas.write(&RawString(&self.bytes))?;
    canvas.set_char(")");

    Ok(())
  }
}
