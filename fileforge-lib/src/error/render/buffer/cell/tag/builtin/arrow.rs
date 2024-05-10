use core::fmt::Write;

use crate::error::render::{buffer::cell::tag::{context::CellTagContext, CellTag}, grapheme::Grapheme};

pub struct Cradle;

pub static CRADLE: Cradle = Cradle;

impl CellTag for Cradle {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<Cradle>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[38;5;240m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str { "fileforge-lib-builtin-tag-cradle" }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) { (85, 85, 85, 255) }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> { Ok(()) }
}

pub struct ArrowBody;

pub static ARROW_BODY: ArrowBody = ArrowBody;

impl CellTag for ArrowBody {
  fn get_name(&self) -> &'static str {
    core::any::type_name::<ArrowBody>()
  }

  fn get_ansi_color(&self, _: Grapheme, _: CellTagContext) -> &'static str { "\x1b[38;5;240m" }
  fn get_html_class_name(&self, _: Grapheme, _: CellTagContext) -> &'static str { "fileforge-lib-builtin-tag-arrow-body" }
  fn get_rgba_color(&self, _: Grapheme, _: CellTagContext) -> (u8, u8, u8, u8) { (85, 85, 85, 255) }

  fn write_hover_text(&self, _: &mut dyn Write, _: Grapheme, _: CellTagContext) -> Result<(), core::fmt::Error> { Ok(()) }
}