use super::ReportNote;

#[derive(Default)]
pub struct ReportNoteSet<'t, 'l, 'pool> {
  pub(crate) notes: heapless::Vec<ReportNote<'t, 'l, 'pool>, 0x10>,
}

impl<'t, 'l, 'pool> ReportNoteSet<'t, 'l, 'pool> {
  pub fn add(
    &mut self,
    note: ReportNote<'t, 'l, 'pool>,
  ) -> Result<(), ReportNote<'t, 'l, 'pool>> {
    self.notes.push(note)
  }
}
