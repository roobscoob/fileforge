use core::fmt::Debug;

use self::{
  render::{
    buffer::{
      cell::{tag::context::RenderMode, RenderBufferCell},
      RenderBuffer,
    },
    position::RenderPosition,
  },
  report::Report,
};

pub mod render;
pub mod report;

pub trait Error<const NODE_NAME_SIZE: usize> {
  fn with_report<Cb: FnMut(Report<NODE_NAME_SIZE>) -> ()>(&self, callback: Cb);

  #[cfg(feature = "alloc")]
  fn into_display(self) -> DisplayableError<NODE_NAME_SIZE, Self>
  where
    Self: Sized,
  {
    DisplayableError(self)
  }
}

#[cfg(feature = "alloc")]
pub struct DisplayableError<
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  E: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
>(pub E);

#[cfg(feature = "alloc")]
impl<const DIAGNOSTIC_NODE_NAME_SIZE: usize, E: Error<DIAGNOSTIC_NODE_NAME_SIZE>> Debug
  for DisplayableError<DIAGNOSTIC_NODE_NAME_SIZE, E>
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut result = Ok(());

    f.write_str("\n")?;

    self.0.with_report(|report| {
      let mut buffer = RenderBuffer::dry();
      let mut canvas = buffer.canvas_at(RenderPosition::zero());
      canvas.write(&report).unwrap();

      let width = buffer.highest_written_column + 1;

      let mut vec = alloc::vec![RenderBufferCell::default(); width];
      let mut slice = vec.as_mut_slice();
      let mut i = 0;

      loop {
        let mut buffer = RenderBuffer::new(&mut slice, width, i);
        let mut canvas = buffer.canvas_at(RenderPosition::zero());

        canvas.write(&report).unwrap();

        if buffer.is_empty() {
          break;
        }

        result = buffer.flush_into(f, RenderMode::TerminalAnsi);

        if result.is_err() {
          return;
        }

        for cell in slice.iter_mut() {
          cell.clear();
        }

        i += 1;
      }
    });

    result
  }
}
