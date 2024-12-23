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

  fn into_display(self) -> DisplayableError<NODE_NAME_SIZE, Self>
  where
    Self: Sized,
  {
    DisplayableError(self)
  }
}

pub trait ErrorResultExt<const NODE_NAME_SIZE: usize, T, E: Error<NODE_NAME_SIZE>> {
  fn unwrap_displayable(self) -> T;
}

impl<const NODE_NAME_SIZE: usize, T, E: Error<NODE_NAME_SIZE>> ErrorResultExt<NODE_NAME_SIZE, T, E>
  for Result<T, E>
{
  fn unwrap_displayable(self) -> T { self.map_err(|e| e.into_display()).unwrap() }
}

pub struct DisplayableError<
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  E: Error<DIAGNOSTIC_NODE_NAME_SIZE>,
>(pub E);

struct S<'a> {
  #[cfg(feature = "alloc")]
  c: alloc::vec::Vec<RenderBufferCell<'a>>,

  #[cfg(not(feature = "alloc"))]
  c: [RenderBufferCell<'a>; 80],
}

impl<const DIAGNOSTIC_NODE_NAME_SIZE: usize, E: Error<DIAGNOSTIC_NODE_NAME_SIZE>> Debug
  for DisplayableError<DIAGNOSTIC_NODE_NAME_SIZE, E>
{
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut result = Ok(());

    f.write_str("\n")?;

    self.0.with_report(|report| {
      #[cfg(feature = "alloc")]
      let mut slice: S = {
        let mut buffer = RenderBuffer::dry();
        let mut canvas = buffer.canvas_at(RenderPosition::zero());
        canvas.write(&report).unwrap();

        let width = buffer.highest_written_column + 1;
        S {
          c: alloc::vec![RenderBufferCell::default(); width],
        }
      };

      #[cfg(not(feature = "alloc"))]
      let mut slice: S = {
        S {
          c: [RenderBufferCell::default(); 80],
        }
      };

      let mut slice = slice.c.as_mut_slice();
      let mut i = 0;

      loop {
        let len = slice.len();
        let mut buffer = RenderBuffer::new(&mut slice, len, i);
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
