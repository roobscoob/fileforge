use super::ReportNote;

#[derive(Default)]
pub struct ReportNoteSet<'t, 'l> {
  pub(crate) notes: heapless::Vec<ReportNote<'t, 'l>, 0x10>,
}

impl<'t, 'l> ReportNoteSet<'t, 'l> {
  pub fn add(&mut self, note: ReportNote<'t, 'l>) -> Result<(), ReportNote<'t, 'l>> {
    self.notes.push(note)
  }
}
