use core::{fmt::Debug, ops::Deref};

use fileforge_lib::{
  diagnostic::node::reference::DiagnosticReference,
  error::{
    render::{
      buffer::cell::tag::builtin::report::{REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::{number::formatted_unsigned::FormattedUnsigned, text::Text},
    },
    report::{kind::ReportKind, note::ReportNote, Report},
    Error,
  },
  provider::error::ProviderError,
  reader::endianness::Endianness,
};

use super::get_header::GetHeaderError;

pub enum LoadError<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  HeaderGetError(GetHeaderError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>),
  UnsupportedVersion(
    u16,
    Endianness,
    DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ),
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  LoadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>
{
  pub fn assert_supported<Cb: FnOnce() -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(
    version: u16,
    endianness: Endianness,
    make_dr: Cb,
  ) -> Result<(), LoadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>> {
    if version > 10 {
      Err(Self::UnsupportedVersion(version, endianness, make_dr()))
    } else {
      Ok(())
    }
  }
}

impl<'pool, Re: ProviderError, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  Error<DIAGNOSTIC_NODE_NAME_SIZE> for LoadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>
{
  fn with_report<Cb: FnMut(Report<DIAGNOSTIC_NODE_NAME_SIZE>) -> ()>(&self, mut callback: Cb) {
    match self {
      LoadError::HeaderGetError(e) => e.with_report(callback),
      LoadError::UnsupportedVersion(version, native_endianness, dr) => {
        let version_formatted = FormattedUnsigned::new(*version as u64)
          .with_separator(3, ",")
          .with_tag(&REPORT_INFO_LINE_TEXT);

        let text = Text::new()
          .push("Version ", &REPORT_INFO_LINE_TEXT)
          .with(&version_formatted)
          .push(" not supported", &REPORT_INFO_LINE_TEXT);

        let version_hex = FormattedUnsigned::new(*version as u64)
          .with_base(16)
          .with_padding(4)
          .with_tag(&REPORT_INFO_LINE_TEXT);

        let version_swap_formatted = FormattedUnsigned::new(version.swap_bytes() as u64)
          .with_separator(3, ",")
          .with_tag(&REPORT_INFO_LINE_TEXT);

        let version_swap_hex = FormattedUnsigned::new(version.swap_bytes() as u64)
          .with_base(16)
          .with_padding(4)
          .with_tag(&REPORT_INFO_LINE_TEXT);

        let endianness_swap_flag: Option<Text> = if let Ok(()) =
          LoadError::<Re, DIAGNOSTIC_NODE_NAME_SIZE>::assert_supported(
            version.swap_bytes(),
            *native_endianness,
            || *dr,
          ) {
          // The endianness defined in the header, or the endianness the version was written with might be wrong!
          // When swapping the byte-order of the version from the file-defined Big (0x0300, 768) to Little (0x0003, 3) the version becomes valid!

          Some(Text::new()
            .push("The endianness defined in the header, or the endianness the version was written with might be wrong!\nWhen swapping the byte-order of the version from the file-defined ", &REPORT_FLAG_LINE_TEXT)
            .push(if *native_endianness == Endianness::Little { "Little (0x" } else { "Big (0x" }, &REPORT_FLAG_LINE_TEXT)
            .with(&version_hex)
            .push(", ", &REPORT_FLAG_LINE_TEXT)
            .with(&version_formatted)
            .push(if *native_endianness == Endianness::Little { ") to Big (0x" } else { ") to Little (0x" }, &REPORT_FLAG_LINE_TEXT)
            .with(&version_swap_hex)
            .push(", ", &REPORT_FLAG_LINE_TEXT)
            .with(&version_swap_formatted)
            .push(") the version becomes valid!", &REPORT_FLAG_LINE_TEXT))
        } else {
          None
        };

        if dr.family_exists() {
          let report = Report::new::<LoadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>(
            ReportKind::Error,
            "Failed to load BYML",
          )
          .with_note(|| {
            ReportNote::new(&text)
              .with_location(*dr, &version_formatted)
              .unwrap()
          })
          .unwrap();

          if let Some(sf) = endianness_swap_flag {
            callback(report.with_flag_line(&sf).unwrap());
          } else {
            callback(report);
          };
        } else {
          let line = Text::new()
            .push("The diagnostic pool was too small to be able to load the diagnostics for this error. You are seeing a minified version with what available data exists.", &REPORT_FLAG_LINE_TEXT);

          let report = Report::new::<LoadError<'pool, Re, DIAGNOSTIC_NODE_NAME_SIZE>>(
            ReportKind::Error,
            "Failed to load BYML",
          )
          .with_flag_line(&line)
          .unwrap()
          .with_info_line(&text)
          .unwrap();

          if let Some(sf) = endianness_swap_flag {
            callback(report.with_flag_line(&sf).unwrap());
          } else {
            callback(report);
          };
        }
      }
    }
  }
}
