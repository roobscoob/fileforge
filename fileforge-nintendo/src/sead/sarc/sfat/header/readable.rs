use fileforge_macros::story;
use fileforge::{
  binary_reader::{
    error::{primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  diagnostic::value::{DiagnosticSaturation, DiagnosticValue},
  error::{ext::annotations::annotated::Annotated, FileforgeError},
  stream::{error::user_read::UserReadError, ReadableStream},
};
use fileforge_std::magic::{Magic, MagicError};

use super::{SfatHeader, SFAT_HEADER_SIZE, SFAT_MAX_FILES};
use crate::report::{render_field_read, render_magic, render_wrong_value, Field, MagicMeaning, CORRUPTED};
use fileforge::error::{
  render::{
    buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
    builtin::number::formatted_unsigned::FormattedUnsigned,
  },
  report::{note::ReportNote, Report},
};
use fileforge_macros::text;

pub const SFAT_MAGIC: Magic<4> = Magic::from_byte_ref(b"SFAT");

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SfatHeader {
  type Error = SfatHeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(SFAT_MAGIC).await.map_err(|e| SfatHeaderReadError::Magic(e))?;

    let header_length: u16 = reader.get().await.map_err(|e| SfatHeaderReadError::HeaderLength(e))?;

    if header_length != SFAT_HEADER_SIZE {
      return Err(SfatHeaderReadError::WrongHeaderLength(reader.create_physical_diagnostic(-2, Some(2), "HeaderLength").saturate(header_length)));
    }

    let file_count: u16 = reader.get().await.map_err(|e| SfatHeaderReadError::FileCount(e))?;

    if file_count > SFAT_MAX_FILES {
      return Err(SfatHeaderReadError::TooManyFiles(reader.create_physical_diagnostic(-2, Some(2), "FileCount").saturate(file_count)));
    }

    Ok(SfatHeader {
      file_count,
      hash_multiplier: reader.get().await.map_err(|e| SfatHeaderReadError::HashMultiplier(e))?,
    })
  }
}

#[story("invalid magic", SfatHeaderReadError::Magic({
  let error: fileforge_std::magic::MagicError<'_, 4, StoryUserError> = fileforge_std::magic::MagicError::Invalid {
    actual: dv!("archive.sarc" / "Magic" @ 0..4 [fileforge_std::magic::Magic::from_bytes(*b"BAD!")]),
    expected: fileforge_std::magic::Magic::from_bytes(*b"SFAT"),
  };
  error
}))]
#[story("file ends inside the header length", SfatHeaderReadError::HeaderLength(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..25), 24, DiagnosticValue(25, None))))]
#[story("wrong header length", SfatHeaderReadError::<StoryUserError>::WrongHeaderLength(dv!("archive.sarc" / "HeaderLength" @ 24..26 [0x10u16])))]
#[story("too many files", SfatHeaderReadError::<StoryUserError>::TooManyFiles(dv!("archive.sarc" / "FileCount" @ 26..28 [0x4000u16])))]
#[story("file ends inside the file count", SfatHeaderReadError::FileCount(read_exhausted::<u16, StoryUserError>(dr!("archive.sarc" @ 0..27), 26, DiagnosticValue(27, None))))]
#[story("file ends inside the hash multiplier", SfatHeaderReadError::HashMultiplier(read_exhausted::<u32, StoryUserError>(dr!("archive.sarc" @ 0..30), 28, DiagnosticValue(30, None))))]
pub enum SfatHeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  HeaderLength(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  FileCount(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  HashMultiplier(Annotated<PrimitiveName<fileforge::binary_reader::error::common::Read>, GetPrimitiveError<'pool, U>>),
  /// The header length isn't [`SFAT_HEADER_SIZE`].
  WrongHeaderLength(DiagnosticValue<'pool, u16>),
  /// The file count is over [`SFAT_MAX_FILES`].
  TooManyFiles(DiagnosticValue<'pool, u16>),
}

impl<'pool, U: UserReadError> FileforgeError for SfatHeaderReadError<'pool, U> {
  fn render_into_report<P: fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
    &self,
    provider: P,
    callback: impl for<'tag, 'b> FnOnce(fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
  ) {
    let field = |name, kind| Field { structure: "SFAT header", name, kind };

    match self {
      Self::HeaderLength(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("header length", "a u16"), provider, callback),
      Self::FileCount(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("file count", "a u16"), provider, callback),
      Self::WrongHeaderLength(actual) => {
        render_wrong_value::<Self, u16, P, ITEM_NAME_SIZE>(actual, SFAT_HEADER_SIZE, field("length", "a u16"), "Invalid SFAT header", provider, callback)
      }
      Self::TooManyFiles(actual) => {
        let count = FormattedUnsigned::new(**actual as u128).separator(3, ",");
        let max = FormattedUnsigned::new(SFAT_MAX_FILES as u128).separator(3, ",");

        let count_text = text!([&REPORT_ERROR_TEXT] "The archive lists {&count} files, but a SARC archive can hold at most {&max}.");
        let note_text = text!([&REPORT_INFO_LINE_TEXT] "The file count");
        let location = actual.map(|count| FormattedUnsigned::new(count as u128));

        let mut report = Report::new::<Self>(provider, &"Too many files in SARC archive").with_info_line(&count_text).with_flag_line(&CORRUPTED);

        if location.reference().is_some() {
          report.add_note(ReportNote::new(&note_text).with_location(&location).with_tag(&REPORT_INFO_LINE_TEXT));
        }

        report.apply(callback);
      }
      Self::HashMultiplier(read) => render_field_read::<Self, U, P, ITEM_NAME_SIZE>(read, field("hash multiplier", "a u32"), provider, callback),
      Self::Magic(error) => render_magic::<Self, 4, U, P, ITEM_NAME_SIZE>(
        error,
        MagicMeaning { structure: "SFAT header", title: "Missing SFAT section", subject: "An SFAT section" },
        provider,
        callback,
      ),
    }
  }
}
