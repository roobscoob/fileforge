use fileforge_macros::text;

use crate::{diagnostic::node::{name::DiagnosticNodeName, reference::DiagnosticReference, DiagnosticNode}, error::{render::{buffer::cell::tag::{builtin::report::{REPORT_ERROR_TEXT, REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT, REPORT_WARNING_TEXT}, CellTag}, builtin::{number::formatted_unsigned::FormattedUnsigned, text::Text}, r#trait::renderable::Renderable}, report::{kind::ReportKind, note::ReportNote, Report}, Error}, object::nintendo::byml::unmanaged::string_table::{error::size::StringTableSizeError, DiagnosticReferenceBuilder}, provider::{error::ProviderError, out_of_bounds::SliceOutOfBoundsError}};

use super::get_header::GetHeaderError;

pub enum GetStringTableError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  GetHeaderError(GetHeaderError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  GetStringTableSizeError(StringTableSizeError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  StringTableOutOfBounds(StringTableOutOfBounds<'pool, DIAGNOSTIC_NODE_NAME_SIZE>),
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for GetStringTableError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(crate::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, callback: Cb) {
    match self {
      GetStringTableError::GetHeaderError(ghe) => ghe.with_report(callback),
      GetStringTableError::GetStringTableSizeError(gstse) => gstse.with_report(callback),
      GetStringTableError::StringTableOutOfBounds(stoob) => stoob.with_report(callback),
    }
  }
}

pub struct StringTableOutOfBounds<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  pub string_table_size: Option<usize>,
  pub string_table_size_complete: bool,
  pub string_table_position: usize,
  pub string_table_parent: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  pub string_table_size_dr: Option<DiagnosticReferenceBuilder<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>,
  pub string_table_position_dr: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  pub byml_size: usize,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> Error<DIAGNOSTIC_NODE_NAME_SIZE> for StringTableOutOfBounds<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
  fn with_report<Cb: FnMut(crate::error::report::Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    let string_table_dr = self.string_table_parent.create_physical_child(self.string_table_position as u64, self.string_table_size.unwrap_or(0) as u64, DiagnosticNodeName::from("String Table"));
    let string_table_size_dr = self.string_table_size_dr.as_ref().map(|v| v.relocate_build(string_table_dr)).flatten();

    let minified = &text!([&REPORT_FLAG_LINE_TEXT] "The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.");
    let primary_note_width = self.string_table_size.map(|size| FormattedUnsigned::new(size as u64));
    let mut primary_note_position = FormattedUnsigned::new(self.string_table_position as u64);

    if primary_note_width.is_some() {
      primary_note_position = primary_note_position.with_base(16).with_prefix("0x");
    }

    let primary_note_actual = FormattedUnsigned::new(if primary_note_width.is_some() { self.byml_size - self.string_table_position } else { self.byml_size } as u64);

    let primary_note = &text!(
      { desired.is_some() && !self.string_table_size_complete }
      [&REPORT_ERROR_TEXT] "BYML was not large enough for string table. Expected at least {desired.as_ref().unwrap()} bytes at {position}, but only {actual} were available.",
      { desired.is_some() }
      [&REPORT_ERROR_TEXT] "BYML was not large enough for string table. Expected exactly {desired.as_ref().unwrap()} bytes at {position}, but only {actual} were available.",
      [&REPORT_ERROR_TEXT] "BYML was not large enough for string table. Expected at least {position} bytes, but only {actual} were available.",
      desired = &primary_note_width,
      position = &primary_note_position,
      actual = &primary_note_actual,
    );

    let size_note_size = self.string_table_size.map(|size| FormattedUnsigned::new(size as u64));
    let size_note_size_raw = &string_table_size_dr.map(|(size, _)| FormattedUnsigned::new(size as u64));

    let size_note = &text!(
      { size.is_some() }
      [&REPORT_INFO_LINE_TEXT] "The size was computed to be {size.as_ref().unwrap()}",
      [&REPORT_INFO_LINE_TEXT] "The size failed to compute",
      size = &size_note_size
    );

    let position_note_position = FormattedUnsigned::new(self.string_table_position as u64).with_base(16).with_prefix("0x");

    let position_note = &text!(
      [&REPORT_INFO_LINE_TEXT] "The position was found to be {position}",
      position = &position_note_position
    );

    let mut report = Report::new::<Self>(ReportKind::Error, "String Table Out Of Bounds");

    if !self.string_table_parent.family_exists() || !string_table_size_dr.map(|dr| dr.1.family_exists()).unwrap_or(true) || !self.string_table_position_dr.family_exists() {
      report = report.with_flag_line(minified).unwrap()
    };

    if let Some(size_dr) = string_table_size_dr {
      if size_dr.1.family_exists() {
        report = report.with_note(|| {
          ReportNote::new(size_note)
            .with_location(size_dr.1, size_note_size_raw.as_ref().unwrap()).unwrap()
        }).unwrap();
      }
    }

    if self.string_table_position_dr.family_exists() {
      report = report.with_note(|| {
        ReportNote::new(position_note)
          .with_location(self.string_table_position_dr, &position_note_position).unwrap()
      }).unwrap()
    }

    if self.string_table_parent.family_exists() {
      report = report.with_note(|| {
        ReportNote::new(primary_note)
          .with_unvalued_location(string_table_dr.parent_reference().unwrap()).unwrap()
      }).unwrap();
    } else {
      report = report.with_info_line(primary_note).unwrap()
    }

    callback(report);
  }
}