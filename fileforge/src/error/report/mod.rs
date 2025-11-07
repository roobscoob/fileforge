use crate::diagnostic::pool::DiagnosticPoolProvider;

use self::note::{set::ReportNoteSet, ReportNote};

use super::render::{
  buffer::{
    canvas::RenderBufferCanvas,
    cell::tag::builtin::report::{REPORT_ERROR_HEADER, REPORT_FLAG_LINE_SYMBOL, REPORT_INFO_LINE_SYMBOL, REPORT_INFO_NAME, REPORT_INFO_SYMBOL, REPORT_INFO_TYPENAME, REPORT_INFO_TYPENAME_CELL},
  },
  builtin::diagnostic_info::DiagnosticInfo,
  r#trait::renderable::Renderable,
};

// pub mod kind;
pub mod location;
pub mod note;

pub struct Report<'tag, 'l, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> {
  pool: P,
  // kind: ReportKind,
  info_name: &'l dyn Renderable<'tag>,
  info_typename: &'static str,
  info_lines: heapless::Vec<&'l dyn Renderable<'tag>, 0x10>,
  flag_lines: heapless::Vec<&'l dyn Renderable<'tag>, 0x10>,
  notes: ReportNoteSet<'tag, 'l>,
}

impl<'t, 'l, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> Report<'t, 'l, ITEM_NAME_SIZE, P> {
  pub fn new<T>(provider: P, name: &'l dyn Renderable<'t>) -> Self {
    Report {
      pool: provider,
      // kind,
      info_name: name,
      info_typename: core::any::type_name::<T>(),

      info_lines: Default::default(),
      flag_lines: Default::default(),
      notes: Default::default(),
    }
  }

  pub fn with_info_line(mut self, line: &'l dyn Renderable<'t>) -> Self {
    self.info_lines.push(line).map_err(|_| {}).expect("Info Line Container Full");
    self
  }

  pub fn add_info_line(&mut self, line: &'l dyn Renderable<'t>) {
    self.info_lines.push(line).map_err(|_| {}).expect("Info Line Container Full");
  }

  pub fn with_flag_line(mut self, line: &'l dyn Renderable<'t>) -> Self {
    self.flag_lines.push(line).map_err(|_| {}).expect("Flag Line Container Full");
    self
  }

  pub fn add_flag_line(&mut self, line: &'l dyn Renderable<'t>) {
    self.flag_lines.push(line).map_err(|_| {}).expect("Flag Line Container Full");
  }

  pub fn with_note<Cb: FnOnce() -> ReportNote<'t, 'l>>(mut self, builder: Cb) -> Self {
    self.notes.add(builder()).map_err(|_| {}).expect("Flag Line Container Full");
    self
  }

  pub fn add_note(&mut self, note: ReportNote<'t, 'l>) {
    self.notes.add(note).map_err(|_| {}).expect("Flag Line Container Full");
  }

  pub fn apply(self, into: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    (into)(self)
  }

  pub fn pool<'a>(&'a self) -> &'a P {
    &self.pool
  }
}

impl<'t, 'l, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider> Renderable<'t> for Report<'t, 'l, ITEM_NAME_SIZE, P> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    // match self.kind {
    //   ReportKind::Error => {
    canvas.set_tagged_str("× ", &REPORT_ERROR_HEADER);
    canvas.set_tagged_str("FileForgeLib Error Report", &REPORT_ERROR_HEADER);
    canvas.set_tagged_str(" ×", &REPORT_ERROR_HEADER);
    //   }

    //   ReportKind::Warning => {
    //     canvas.set_tagged_str("× ", &REPORT_WARNING_HEADER);
    //     canvas.set_tagged_str("FileForgeLib Warning Report", &REPORT_WARNING_HEADER);
    //     canvas.set_tagged_str(" ×", &REPORT_WARNING_HEADER);
    //   }
    // }

    canvas.cursor_down().cursor_down().set_column(0);

    canvas.set_tagged_str("i ", &REPORT_INFO_SYMBOL);
    let indent = canvas.get_position().column();
    canvas.write_tagged(self.info_name, &REPORT_INFO_NAME)?;
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

    DiagnosticInfo::transform_diagnostics::<_, ITEM_NAME_SIZE>(&self.pool, &self.notes, |root| {
      canvas.write(root).unwrap();
    });

    Ok(())
  }
}
