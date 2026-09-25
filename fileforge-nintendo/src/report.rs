//! Report pieces shared by the formats in this crate.

use fileforge::{
  binary_reader::error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    ext::annotations::annotated::Annotated,
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::{bytes::Bytes, number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
    },
    report::{note::ReportNote, Report},
    FileforgeError,
  },
  stream::error::user_read::UserReadError,
};
use fileforge_macros::text;
use fileforge_std::magic::{MagicError, WRONG_FORMAT};

pub(crate) const TRUNCATED: ConstText = ConstText::new(
  "The file is probably incomplete, for example because a download or an extraction was interrupted.",
  &REPORT_FLAG_LINE_TEXT,
);

pub(crate) const CORRUPTED: ConstText = ConstText::new("This usually means the file is corrupted.", &REPORT_FLAG_LINE_TEXT);

const TRUNCATED_FILE_NOTE: ConstText = ConstText::new("This file is truncated", &REPORT_ERROR_TEXT);

/// A field of a structure, for reports about reading it.
#[derive(Clone, Copy)]
pub(crate) struct Field {
  /// The structure the field belongs to, such as "SARC header".
  pub structure: &'static str,
  /// The field's name, such as "file size".
  pub name: &'static str,
  /// What the field holds, such as "a u32" or "4 bytes".
  pub kind: &'static str,
}

/// Reports a failed read of `field`. If the data ran out partway through the structure, says so
/// in terms of that field; any other failure shows the underlying report and where it happened.
///
/// `E` is the error being reported, whose name the report shows.
pub(crate) fn render_field_read<'pool, E, U: UserReadError, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
  read: &Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>>,
  field: Field,
  provider: P,
  callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
) {
  let Field { structure, name, kind } = field;

  let GetPrimitiveError::ReaderExhausted(exhausted) = read.error() else {
    // Not a failure we understand: show the underlying error's own report, and say where it
    // happened instead of only which type was being read.
    return read.error().render_into_report(provider, move |report| {
      let context = text!([&REPORT_INFO_LINE_TEXT] "This happened while reading the {&name} ({&kind}) of the {&structure}.");
      report.with_info_line(&context).apply(callback)
    });
  };

  let file_length = *exhausted.stream_length;
  let needed = *exhausted.length;
  let remaining = file_length.saturating_sub(exhausted.offset);
  let required_length = exhausted.offset + needed;

  let offset = FormattedUnsigned::new(exhausted.offset as u128).base(16).uppercase().prefix("0x");
  let needed_text = FormattedUnsigned::new(needed as u128);
  let remaining_text = FormattedUnsigned::new(remaining as u128);
  let file_length_text = FormattedUnsigned::new(file_length as u128).separator(3, ",");
  let file_length_hex = FormattedUnsigned::new(file_length as u128).base(16).uppercase().prefix("0x");
  let required_text = FormattedUnsigned::new(required_length as u128).separator(3, ",");
  let required_hex = FormattedUnsigned::new(required_length as u128).base(16).uppercase().prefix("0x");

  let title = text!("Truncated {&structure}");

  let ends_text = text!(
    { remaining == 0 }
      [&REPORT_ERROR_TEXT] "The file ends right before the {&name} ({&kind} at {&offset}).",

    [&REPORT_ERROR_TEXT] "The file ends partway through the {&name} ({&kind} at {&offset})."
  );

  let remaining_line = text!(
    { remaining == 1 }
      [&REPORT_INFO_LINE_TEXT] "It needs {&needed_text} bytes there, but only 1 remains.",

    [&REPORT_INFO_LINE_TEXT] "It needs {&needed_text} bytes there, but only {&remaining_text} remain."
  );

  let length_text = text!(
    { file_length == 1 }
      [&REPORT_INFO_LINE_TEXT] "The file is 1 byte long, but at least {&required_text} ({&required_hex}) are needed to include the {&name}.",

    [&REPORT_INFO_LINE_TEXT] "The file is {&file_length_text} ({&file_length_hex}) bytes long, but at least {&required_text} ({&required_hex}) are needed to include the {&name}."
  );

  let length_source = exhausted.stream_length.map(|length| FormattedUnsigned::new(length as u128).base(16).uppercase().prefix("0x"));
  let length_source_text = text!([&REPORT_INFO_LINE_TEXT] "The file's length of {&file_length_text} bytes comes from here, so this value may be the real problem.");

  let mut report = Report::new::<E>(provider, &title).with_info_line(&ends_text);

  if remaining != 0 {
    report.add_info_line(&remaining_line);
  }

  let mut report = report.with_info_line(&length_text).with_flag_line(&TRUNCATED);

  if let Some(container) = exhausted.container {
    report.add_note(ReportNote::new(&TRUNCATED_FILE_NOTE).with_location(container).with_tag(&REPORT_ERROR_TEXT));
  }

  if length_source.reference().is_some() {
    report.add_note(ReportNote::new(&length_source_text).with_location(&length_source).with_tag(&REPORT_INFO_LINE_TEXT));
  }

  report.apply(callback);
}

/// What a structure's magic says about the data, for reports about a wrong one.
#[derive(Clone, Copy)]
pub(crate) struct MagicMeaning {
  /// The structure the magic belongs to, such as "SARC header".
  pub structure: &'static str,
  /// The report's title when the magic is wrong, such as "Not a SARC archive".
  pub title: &'static str,
  /// What starts with the magic, such as "A SARC archive".
  pub subject: &'static str,
}

/// Reports a failed read of a structure's magic. A wrong magic is reported as the data not being
/// that structure at all; a failed read is reported like any other field.
pub(crate) fn render_magic<'pool, E, const SIZE: usize, U: UserReadError, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
  error: &MagicError<'pool, SIZE, U>,
  meaning: MagicMeaning,
  provider: P,
  callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
) {
  let MagicMeaning { structure, title, subject } = meaning;

  let (actual, expected) = match error {
    MagicError::Failed(read) => {
      let kind = match SIZE {
        2 => "2 bytes",
        4 => "4 bytes",
        _ => "bytes",
      };

      return render_field_read::<E, U, P, ITEM_NAME_SIZE>(read, Field { structure, name: "magic", kind }, provider, callback);
    }
    MagicError::Invalid { actual, expected } => (actual, expected),
  };

  let found = Bytes(actual.bytes());
  let expected = Bytes(expected.bytes());

  let found_text = text!([&REPORT_ERROR_TEXT] "Found {&found}.");
  let expected_text = text!([&REPORT_INFO_LINE_TEXT] "{&subject} starts with {&expected}.");
  let note_text = text!([&REPORT_INFO_LINE_TEXT] "Expected {&expected} here");

  let location = actual.map(|magic| Bytes(magic.bytes()));

  let mut report = Report::new::<E>(provider, &title)
    .with_info_line(&found_text)
    .with_info_line(&expected_text)
    .with_flag_line(&WRONG_FORMAT);

  if location.reference().is_some() {
    report.add_note(ReportNote::new(&note_text).with_location(&location).with_tag(&REPORT_INFO_LINE_TEXT));
  }

  report.apply(callback);
}

/// Renders `error`'s own report with one more line saying where it happened.
pub(crate) fn render_with_context<E: FileforgeError, P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(
  error: &E,
  context: &'static ConstText,
  provider: P,
  callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> (),
) {
  error.render_into_report(provider, |report| report.with_info_line(context).apply(callback))
}
