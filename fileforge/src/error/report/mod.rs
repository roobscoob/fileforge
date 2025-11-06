use crate::diagnostic::pool::DiagnosticPoolProvider;

use self::{
  kind::ReportKind,
  note::{set::ReportNoteSet, ReportNote},
};

use super::render::{
  buffer::{
    canvas::RenderBufferCanvas,
    cell::tag::builtin::report::{
      REPORT_ERROR_HEADER, REPORT_FLAG_LINE_SYMBOL, REPORT_INFO_LINE_SYMBOL, REPORT_INFO_NAME,
      REPORT_INFO_SYMBOL, REPORT_INFO_TYPENAME, REPORT_INFO_TYPENAME_CELL, REPORT_WARNING_HEADER,
    },
  },
  builtin::diagnostic_info::DiagnosticInfo,
  r#trait::renderable::Renderable,
};

pub mod kind;
pub mod location;
pub mod note;

pub struct Report<'tag, 'l, 'pool, 'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> {
  pool: &'pool_ref P,
  kind: ReportKind,
  info_name: &'static str,
  info_typename: &'static str,
  info_lines: heapless::Vec<&'l dyn Renderable<'tag>, 0x10>,
  flag_lines: heapless::Vec<&'l dyn Renderable<'tag>, 0x10>,
  notes: ReportNoteSet<'tag, 'l, 'pool>,
}

impl<'t, 'l, 'pool, 'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> Report<'t, 'l, 'pool, 'pool_ref, ITEM_NAME_SIZE, P> {
  pub fn new<T>(provider: &'pool_ref P, kind: ReportKind, name: &'static str) -> Self {
    Report {
      pool: provider,
      kind,
      info_name: name,
      info_typename: core::any::type_name::<T>(),

      info_lines: Default::default(),
      flag_lines: Default::default(),
      notes: Default::default(),
    }
  }

  pub fn with_info_line(mut self, line: &'l dyn Renderable<'t>) -> Result<Self, ()> {
    self.info_lines.push(line).map_err(|_| {})?;
    Ok(self)
  }

  pub fn add_info_line(&mut self, line: &'l dyn Renderable<'t>) -> Result<(), ()> {
    self.info_lines.push(line).map_err(|_| {})?;
    Ok(())
  }

  pub fn with_flag_line(mut self, line: &'l dyn Renderable<'t>) -> Result<Self, ()> {
    self.flag_lines.push(line).map_err(|_| {})?;
    Ok(self)
  }

  pub fn add_flag_line(&mut self, line: &'l dyn Renderable<'t>) -> Result<(), ()> {
    self.flag_lines.push(line).map_err(|_| {})?;
    Ok(())
  }

  pub fn with_note<Cb: FnOnce() -> ReportNote<'t, 'l, 'pool>>(
    mut self,
    builder: Cb,
  ) -> Result<Self, ()> {
    self.notes.add(builder()).map_err(|_| {})?;
    Ok(self)
  }

  pub fn add_note(&mut self, note: ReportNote<'t, 'l, 'pool>) -> Result<(), ()> {
    self.notes.add(note).map_err(|_| {})?;
    Ok(())
  }
}

impl<'t, 'l, 'pool, 'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> Renderable<'t> for Report<'t, 'l, 'pool, 'pool_ref, ITEM_NAME_SIZE, P> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    match self.kind {
      ReportKind::Error => {
        canvas.set_tagged_str("× ", &REPORT_ERROR_HEADER);
        canvas.set_tagged_str("FileForgeLib Error Report", &REPORT_ERROR_HEADER);
        canvas.set_tagged_str(" ×", &REPORT_ERROR_HEADER);
      }

      ReportKind::Warning => {
        canvas.set_tagged_str("× ", &REPORT_WARNING_HEADER);
        canvas.set_tagged_str("FileForgeLib Warning Report", &REPORT_WARNING_HEADER);
        canvas.set_tagged_str(" ×", &REPORT_WARNING_HEADER);
      }
    }

    canvas.cursor_down().cursor_down().set_column(0);

    canvas.set_tagged_str("i ", &REPORT_INFO_SYMBOL);
    let indent = canvas.get_position().column();
    canvas.set_tagged_str(&self.info_name, &REPORT_INFO_NAME);
    canvas.cursor_right();
    canvas.set_tagged_char("(", &REPORT_INFO_TYPENAME_CELL);
    canvas.set_tagged_str(&self.info_typename, &REPORT_INFO_TYPENAME);
    canvas.set_tagged_char(")", &REPORT_INFO_TYPENAME_CELL);
    canvas.cursor_down().set_column(indent);

    for info in self.info_lines.iter() {
      canvas.set_tagged_str("❯ ", &REPORT_INFO_LINE_SYMBOL);
      canvas.write(*info)?;
      canvas.cursor_down().set_column(indent);
    }

    canvas.cursor_down().set_column(0);

    for flag in self.flag_lines.iter() {
      canvas.set_tagged_str("⚑ ", &REPORT_FLAG_LINE_SYMBOL);
      canvas.write(*flag)?;
      canvas.cursor_down().cursor_down().set_column(0);
    }

    DiagnosticInfo::transform_diagnostics::<_, ITEM_NAME_SIZE>(self.pool, &self.notes, |root| {
      canvas.write(root).unwrap();
    });

    Ok(())
  }
}
