use core::fmt::Write;

use crate::error::render::{
  buffer::cell::tag::{context::CellTagContext, CellTag},
  grapheme::Grapheme,
};

pub struct DiagnosticInfoName;

pub static DIAGNOSTIC_INFO_NAME: DiagnosticInfoName = DiagnosticInfoName;

impl CellTag for DiagnosticInfoName {
  fn get_name(&self) -> &'static str { core::any::type_name::<DiagnosticInfoName>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[37m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-diagnostic-info-name"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) {
    (255, 255, 255, 255)
  }

  fn write_hover_text(
    &self,
    _: &mut dyn Write,
    _: Grapheme,
    _: CellTagContext,
  ) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct DiagnosticLocationSeparator;

pub static DIAGNOSTIC_LOCATION_SEPARATOR: DiagnosticLocationSeparator = DiagnosticLocationSeparator;

impl CellTag for DiagnosticLocationSeparator {
  fn get_name(&self) -> &'static str { core::any::type_name::<DiagnosticLocationSeparator>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[38;5;240m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-diagnostic-location-separator"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) { (85, 85, 85, 255) }

  fn write_hover_text(
    &self,
    _: &mut dyn Write,
    _: Grapheme,
    _: CellTagContext,
  ) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct DiagnosticLocation;

pub static DIAGNOSTIC_LOCATION: DiagnosticLocation = DiagnosticLocation;

impl CellTag for DiagnosticLocation {
  fn get_name(&self) -> &'static str { core::any::type_name::<DiagnosticLocation>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[38;5;240m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-diagnostic-location"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) { (85, 85, 85, 255) }

  fn write_hover_text(
    &self,
    _: &mut dyn Write,
    _: Grapheme,
    _: CellTagContext,
  ) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct DiagnosticSeparator;

pub static DIAGNOSTIC_SEPARATOR: DiagnosticSeparator = DiagnosticSeparator;

impl CellTag for DiagnosticSeparator {
  fn get_name(&self) -> &'static str { core::any::type_name::<DiagnosticSeparator>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[38;5;240m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-diagnostic-separator"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) { (85, 85, 85, 255) }

  fn write_hover_text(
    &self,
    _: &mut dyn Write,
    _: Grapheme,
    _: CellTagContext,
  ) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}

pub struct DiagnosticValueSeparator;

pub static DIAGNOSTIC_VALUE_SEPARATOR: DiagnosticValueSeparator = DiagnosticValueSeparator;

impl CellTag for DiagnosticValueSeparator {
  fn get_name(&self) -> &'static str { core::any::type_name::<DiagnosticValueSeparator>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[38;5;240m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-diagnostic-value-separator"
  }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) { (85, 85, 85, 255) }

  fn write_hover_text(
    &self,
    _: &mut dyn Write,
    _: Grapheme,
    _: CellTagContext,
  ) -> Result<(), core::fmt::Error> {
    Ok(())
  }
}
