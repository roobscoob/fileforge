use super::ReportNote;

#[derive(Default)]
pub struct ReportNoteSet<'t, 'l, 'pool, const NODE_NAME_SIZE: usize> {
  pub(crate) notes: heapless::Vec<ReportNote<'t, 'l, 'pool, NODE_NAME_SIZE>, 0x10>,
}

impl<'t, 'l, 'pool, const NODE_NAME_SIZE: usize> ReportNoteSet<'t, 'l, 'pool, NODE_NAME_SIZE> {
  pub fn add(
    &mut self,
    note: ReportNote<'t, 'l, 'pool, NODE_NAME_SIZE>,
  ) -> Result<(), ReportNote<'t, 'l, 'pool, NODE_NAME_SIZE>> {
    self.notes.push(note)
  }
}
