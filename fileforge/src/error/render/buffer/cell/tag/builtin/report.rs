use core::fmt::Write;

use crate::error::render::{
  buffer::cell::tag::{context::CellTagContext, CellTag},
  grapheme::Grapheme,
};

pub struct ReportErrorHeader;

pub const REPORT_ERROR_HEADER: ReportErrorHeader = ReportErrorHeader;

impl CellTag for ReportErrorHeader {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportErrorHeader>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;9m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-error-header"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (224, 102, 102, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportErrorText;

pub const REPORT_ERROR_TEXT: ReportErrorText = ReportErrorText;

impl CellTag for ReportErrorText {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportErrorText>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;9m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-error-text"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (224, 102, 102, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportWarningHeader;

pub const REPORT_WARNING_HEADER: ReportWarningHeader = ReportWarningHeader;

impl CellTag for ReportWarningHeader {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportWarningHeader>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;220m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-warning-header"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (241, 194, 50, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportWarningText;

pub const REPORT_WARNING_TEXT: ReportWarningText = ReportWarningText;

impl CellTag for ReportWarningText {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportWarningText>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;220m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-warning-text"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (241, 194, 50, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportInfoSymbol;

pub const REPORT_INFO_SYMBOL: ReportInfoSymbol = ReportInfoSymbol;

impl CellTag for ReportInfoSymbol {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportInfoSymbol>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;75m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-info-symbol"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (109, 158, 235, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportInfoName;

pub const REPORT_INFO_NAME: ReportInfoName = ReportInfoName;

impl CellTag for ReportInfoName {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportInfoName>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;75m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-info-name"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (109, 158, 235, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportInfoTypename;

pub const REPORT_INFO_TYPENAME: ReportInfoTypename = ReportInfoTypename;

impl CellTag for ReportInfoTypename {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportInfoTypename>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;240m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-info-typename"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (85, 85, 85, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportInfoTypenameCell;

pub const REPORT_INFO_TYPENAME_CELL: ReportInfoTypenameCell = ReportInfoTypenameCell;

impl CellTag for ReportInfoTypenameCell {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportInfoTypenameCell>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;240m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-info-typename-cell"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (85, 85, 85, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportInfoLineSymbol;

pub const REPORT_INFO_LINE_SYMBOL: ReportInfoLineSymbol = ReportInfoLineSymbol;

impl CellTag for ReportInfoLineSymbol {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportInfoLineSymbol>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;75m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-info-line-symbol"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (109, 158, 235, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportInfoLineText;

pub const REPORT_INFO_LINE_TEXT: ReportInfoLineText = ReportInfoLineText;

impl CellTag for ReportInfoLineText {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportInfoLineText>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;75m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-info-line-text"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (109, 158, 235, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportFlagLineSymbol;

pub const REPORT_FLAG_LINE_SYMBOL: ReportFlagLineSymbol = ReportFlagLineSymbol;

impl CellTag for ReportFlagLineSymbol {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportFlagLineSymbol>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;220m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-flag-line-symbol"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (241, 194, 50, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct ReportFlagLineText;

pub const REPORT_FLAG_LINE_TEXT: ReportFlagLineText = ReportFlagLineText;

impl CellTag for ReportFlagLineText {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ReportFlagLineText>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "\x1b[38;5;220m"
  }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-report-flag-line-text"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (241, 194, 50, 255)
  }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}
