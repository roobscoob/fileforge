use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  diagnostic::{
    pool::DiagnosticPoolProvider,
    value::{DiagnosticSaturation, DiagnosticValue},
  },
  error::{
    ext::annotations::annotated::Annotated,
    render::{
      buffer::{
        canvas::RenderBufferCanvas,
        cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT},
      },
      builtin::{bytes::Bytes, number::formatted_unsigned::FormattedUnsigned, raw_string::RawString, text::r#const::ConstText},
      r#trait::renderable::Renderable,
    },
    report::{note::ReportNote, Report},
    FileforgeError,
  },
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_macros::{story, text};

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

#[story("read failed", MagicError::<4, StoryUserError>::Failed(read_exhausted::<[u8; 4], StoryUserError>(dr!("archive.sarc" @ 0..3), 0, DiagnosticValue(3, None))))]
#[story("invalid, with diagnostics", {
  let error: MagicError<'_, 4, StoryUserError> = MagicError::Invalid {
    actual: dv!("archive.sarc" / "Magic" @ 0..4 [Magic::from_bytes(*b"BAD!")]),
    expected: Magic::from_bytes(*b"SARC"),
  };
  error
})]
#[story("invalid, no diagnostics", {
  let error: MagicError<'_, 4, StoryUserError> = MagicError::Invalid {
    actual: DiagnosticValue(Magic::from_bytes(*b"BAD!"), None),
    expected: Magic::from_bytes(*b"SARC"),
  };
  error
})]
pub enum MagicError<'pool, const MAGIC_SIZE: usize, U: UserReadError> {
  Failed(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>),
  Invalid {
    actual: DiagnosticValue<'pool, Magic<MAGIC_SIZE>>,
    expected: Magic<MAGIC_SIZE>,
  },
}

const READING_MAGIC: ConstText = ConstText::new("This happened while reading a magic number.", &REPORT_INFO_LINE_TEXT);

/// The flag shown when data doesn't look like the format being read.
pub const WRONG_FORMAT: ConstText = ConstText::new("This usually means the data isn't in the expected format, or is corrupted.", &REPORT_FLAG_LINE_TEXT);

impl<'pool, const MAGIC_SIZE: usize, U: UserReadError> FileforgeError for MagicError<'pool, MAGIC_SIZE, U> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Failed(read) => read.error().render_into_report(provider, |report| report.with_info_line(&READING_MAGIC).apply(callback)),
      Self::Invalid { actual, expected } => {
        let found = Bytes(actual.bytes);
        let expected = Bytes(expected.bytes);

        let found_text = text!([&REPORT_ERROR_TEXT] "Found {&found} where {&expected} was expected.");
        let note_text = text!([&REPORT_INFO_LINE_TEXT] "Expected {&expected} here");

        let location = actual.map(|magic| Bytes(magic.bytes));

        let mut report = Report::new::<Self>(provider, &"Invalid magic").with_info_line(&found_text).with_flag_line(&WRONG_FORMAT);

        if location.reference().is_some() {
          report.add_note(ReportNote::new(&note_text).with_location(&location).with_tag(&REPORT_INFO_LINE_TEXT));
        }

        report.apply(callback)
      }
    }
  }
}

impl<'pool, const MAGIC_SIZE: usize, U: UserReadError> From<Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>> for MagicError<'pool, MAGIC_SIZE, U> {
  fn from(value: Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>) -> Self {
    Self::Failed(value)
  }
}

impl<const SIZE: usize> Magic<SIZE> {
  pub const fn from_bytes(bytes: [u8; SIZE]) -> Magic<SIZE> {
    Self { bytes }
  }
  pub const fn from_byte_ref(bytes: &[u8; SIZE]) -> Magic<SIZE> {
    Self { bytes: *bytes }
  }

  pub const fn bytes(&self) -> [u8; SIZE] {
    self.bytes
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
