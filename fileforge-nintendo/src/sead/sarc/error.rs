use fileforge::{
  binary_reader::error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError, SkipError},
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    ext::annotations::annotated::Annotated,
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
    },
    report::Report,
    FileforgeError,
  },
  stream::{error::stream_restore::StreamRestoreError, RestorableStream, StreamReadError},
};
use fileforge_macros::{story, text};

use crate::{
  report::{render_with_context, CORRUPTED},
  sead::sarc::sfat::entry::SfatEntryError,
};

/// A failure reading a SARC archive's files: its entries, names or data.
#[story("no such entry", {
  let error: SarcError<'_, StoryStream> = SarcError::NoSuchEntry { index: 5, file_count: 3 };
  error
})]
#[story("entry ends before it starts", {
  let error: SarcError<'_, StoryStream> = SarcError::EndBeforeStart { index: 1, start: 0x40, end: 0x20 };
  error
})]
#[story("data outside the archive", {
  let error: SarcError<'_, StoryStream> = SarcError::DataOutsideArchive { index: 2, end: 0x1F00, file_size: 0x1000 };
  error
})]
#[story("name outside the name table", {
  let error: SarcError<'_, StoryStream> = SarcError::NameOutsideTable { index: 0, position: 0x140, names_end: 0x100 };
  error
})]
#[story("collision index too large", {
  let error: SarcError<'_, StoryStream> = SarcError::CollisionIndexTooLarge { index: 1, sequence: 4 };
  error
})]
#[story("entries out of order", {
  let error: SarcError<'_, StoryStream> = SarcError::NotSorted { index: 3 };
  error
})]
#[story("wrong collision index", {
  let error: SarcError<'_, StoryStream> = SarcError::WrongCollisionIndex { index: 4, expected: 2, found: 1 };
  error
})]
#[story("unnamed entry shares its hash", {
  let error: SarcError<'_, StoryStream> = SarcError::UnnamedCollision { index: 2 };
  error
})]
#[story("name doesn't match its hash", {
  let error: SarcError<'_, StoryStream> = SarcError::NameHashMismatch { index: 1, stored: 0x1234_5678, computed: 0x9ABC_DEF0 };
  error
})]
pub enum SarcError<'pool, S: RestorableStream<Type = u8>> {
  /// Going back to an earlier part of the archive failed.
  Restore(StreamRestoreError<S::RestoreError>),
  /// Moving forward in the archive failed.
  Skip(SkipError<'pool, S::SkipError>),
  /// Reading a value, such as an entry's hash, failed.
  Read(Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>),
  /// Reading an entry failed.
  Entry(SfatEntryError<'pool, S::ReadError>),
  /// Reading a name failed.
  Name(StreamReadError<S::ReadError>),

  /// There is no entry `index`: the archive has `file_count` files.
  NoSuchEntry { index: u32, file_count: u16 },
  /// Entry `index` ends before it starts.
  EndBeforeStart { index: u32, start: u32, end: u32 },
  /// Entry `index`'s data ends at `end` (from the start of the archive), past its end.
  DataOutsideArchive { index: u32, end: u64, file_size: u32 },
  /// Entry `index`'s name is at `position`, outside the name table, which ends at `names_end`.
  NameOutsideTable { index: u32, position: u64, names_end: u64 },
  /// Entry `index` says it's the `sequence`th with its hash, but too few entries come before it.
  CollisionIndexTooLarge { index: u32, sequence: u8 },

  /// Entry `index`'s hash is smaller than the one before it.
  NotSorted { index: u32 },
  /// Entry `index` is the `expected`th with its hash, but says it's the `found`th.
  WrongCollisionIndex { index: u32, expected: u32, found: u8 },
  /// Entry `index` shares its hash with another, but one of them has no name.
  UnnamedCollision { index: u32 },
  /// Entry `index`'s name hashes to `computed`, but its hash is `stored`.
  NameHashMismatch { index: u32, stored: u32, computed: u32 },
}

const RETURNING: ConstText = ConstText::new("This happened while returning to an earlier part of a SARC archive.", &REPORT_INFO_LINE_TEXT);
const MOVING: ConstText = ConstText::new("This happened while moving to a part of a SARC archive.", &REPORT_INFO_LINE_TEXT);
const READING_VALUE: ConstText = ConstText::new("This happened while reading a SARC archive's file entries.", &REPORT_INFO_LINE_TEXT);
const READING_NAME: ConstText = ConstText::new("This happened while reading a file's name from a SARC archive.", &REPORT_INFO_LINE_TEXT);
const NOT_SORTED: ConstText = ConstText::new("sead finds files with a binary search, so the entries must be sorted by name hash.", &REPORT_INFO_LINE_TEXT);
const UNNAMED: ConstText = ConstText::new("An entry can only leave out its name when no other entry shares its hash.", &REPORT_INFO_LINE_TEXT);
const HASH_MODE: ConstText = ConstText::new(
  "If the archive is from the Wii U, its names may use the older, unsigned hash: try opening it with HashMode::Unsigned.",
  &REPORT_INFO_LINE_TEXT,
);

fn hex(value: u64) -> FormattedUnsigned<'static> {
  FormattedUnsigned::new(value as u128).base(16).uppercase().prefix("0x")
}

impl<'pool, S: RestorableStream<Type = u8>> FileforgeError for SarcError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Restore(error) => render_with_context(error, &RETURNING, provider, callback),
      Self::Skip(error) => render_with_context(error, &MOVING, provider, callback),
      Self::Read(error) => render_with_context(error, &READING_VALUE, provider, callback),
      Self::Entry(error) => error.render_into_report(provider, callback),
      Self::Name(error) => render_with_context(error, &READING_NAME, provider, callback),

      Self::NoSuchEntry { index, file_count } => {
        let index = FormattedUnsigned::new(*index as u128);
        let count = FormattedUnsigned::new(*file_count as u128);
        let text = text!(
          { *file_count == 1 }
            [&REPORT_ERROR_TEXT] "Asked for entry {&index}, but the archive only has 1 file.",

          [&REPORT_ERROR_TEXT] "Asked for entry {&index}, but the archive only has {&count} files."
        );

        Report::new::<Self>(provider, &"No such SARC entry").with_info_line(&text).apply(callback)
      }
      Self::EndBeforeStart { index, start, end } => {
        let index = FormattedUnsigned::new(*index as u128);
        let (start, end) = (hex(*start as u64), hex(*end as u64));
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index}'s data ends at {&end}, before it starts at {&start}.");

        Report::new::<Self>(provider, &"Invalid SARC entry").with_info_line(&text).with_flag_line(&CORRUPTED).apply(callback)
      }
      Self::DataOutsideArchive { index, end, file_size } => {
        let index = FormattedUnsigned::new(*index as u128);
        let (end, file_size) = (hex(*end), hex(*file_size as u64));
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index}'s data runs until {&end}, but the archive ends at {&file_size}.");

        Report::new::<Self>(provider, &"SARC entry outside the archive").with_info_line(&text).with_flag_line(&CORRUPTED).apply(callback)
      }
      Self::NameOutsideTable { index, position, names_end } => {
        let index = FormattedUnsigned::new(*index as u128);
        let (position, names_end) = (hex(*position), hex(*names_end));
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index}'s name is at {&position}, but the name table ends at {&names_end}.");

        Report::new::<Self>(provider, &"SARC name outside the name table").with_info_line(&text).with_flag_line(&CORRUPTED).apply(callback)
      }
      Self::CollisionIndexTooLarge { index, sequence } => {
        let index = FormattedUnsigned::new(*index as u128);
        let sequence = FormattedUnsigned::new(*sequence as u128);
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index} says it's number {&sequence} of the entries with its hash, but there aren't enough entries before it.");

        Report::new::<Self>(provider, &"Invalid SARC collision index").with_info_line(&text).with_flag_line(&CORRUPTED).apply(callback)
      }
      Self::NotSorted { index } => {
        let before = FormattedUnsigned::new(*index as u128 - 1);
        let index = FormattedUnsigned::new(*index as u128);
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index}'s hash is smaller than entry {&before}'s.");

        Report::new::<Self>(provider, &"SARC entries out of order")
          .with_info_line(&text)
          .with_info_line(&NOT_SORTED)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
      Self::WrongCollisionIndex { index, expected, found } => {
        let index = FormattedUnsigned::new(*index as u128);
        let (expected, found) = (FormattedUnsigned::new(*expected as u128), FormattedUnsigned::new(*found as u128));
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index} is number {&expected} of the entries with its hash, but says it's number {&found}.");

        Report::new::<Self>(provider, &"Invalid SARC collision index").with_info_line(&text).with_flag_line(&CORRUPTED).apply(callback)
      }
      Self::UnnamedCollision { index } => {
        let index = FormattedUnsigned::new(*index as u128);
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index} shares its hash with another entry, and one of them has no name.");

        Report::new::<Self>(provider, &"Unnamed SARC entry collides")
          .with_info_line(&text)
          .with_info_line(&UNNAMED)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
      Self::NameHashMismatch { index, stored, computed } => {
        let index = FormattedUnsigned::new(*index as u128);
        let (stored, computed) = (hex(*stored as u64), hex(*computed as u64));
        let text = text!([&REPORT_ERROR_TEXT] "Entry {&index}'s name hashes to {&computed}, but the entry's hash is {&stored}.");

        Report::new::<Self>(provider, &"SARC name doesn't match its hash")
          .with_info_line(&text)
          .with_info_line(&HASH_MODE)
          .with_flag_line(&CORRUPTED)
          .apply(callback)
      }
    }
  }
}

impl<'pool, S: RestorableStream<Type = u8>> From<StreamRestoreError<S::RestoreError>> for SarcError<'pool, S> {
  fn from(value: StreamRestoreError<S::RestoreError>) -> Self {
    Self::Restore(value)
  }
}

impl<'pool, S: RestorableStream<Type = u8>> From<SkipError<'pool, S::SkipError>> for SarcError<'pool, S> {
  fn from(value: SkipError<'pool, S::SkipError>) -> Self {
    Self::Skip(value)
  }
}

impl<'pool, S: RestorableStream<Type = u8>> From<Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>> for SarcError<'pool, S> {
  fn from(value: Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>) -> Self {
    Self::Read(value)
  }
}

impl<'pool, S: RestorableStream<Type = u8>> From<SfatEntryError<'pool, S::ReadError>> for SarcError<'pool, S> {
  fn from(value: SfatEntryError<'pool, S::ReadError>) -> Self {
    Self::Entry(value)
  }
}
