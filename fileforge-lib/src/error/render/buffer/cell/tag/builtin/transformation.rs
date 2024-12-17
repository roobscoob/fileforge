use core::fmt::Write;

use crate::error::render::{
  buffer::cell::tag::{context::CellTagContext, CellTag},
  grapheme::Grapheme,
};

pub struct TransformationName;

pub static TRANSFORMATION_NAME: TransformationName = TransformationName;

impl CellTag for TransformationName {
  fn get_name(&self) -> &'static str { core::any::type_name::<TransformationName>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[37m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-transformation-name"
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

pub struct TransformationSeparator;

pub static TRANSFORMATION_SEPARATOR: TransformationSeparator = TransformationSeparator;

impl CellTag for TransformationSeparator {
  fn get_name(&self) -> &'static str { core::any::type_name::<TransformationSeparator>() }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[37m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str {
    "fileforge-lib-builtin-tag-transformation-separator"
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
