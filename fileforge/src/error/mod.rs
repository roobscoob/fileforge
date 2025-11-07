use render::{
  buffer::{
    cell::{tag::context::RenderMode, RenderBufferCell},
    RenderBuffer,
  },
  position::RenderPosition,
};
use report::Report;

use crate::diagnostic::pool::DiagnosticPoolProvider;

pub mod context;
pub mod ext;
pub mod render;
pub mod report;

pub trait FileforgeError {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ());
}

impl FileforgeError for core::convert::Infallible {
  fn render_into_report<P: DiagnosticPoolProvider, const ITEM_NAME_SIZE: usize>(&self, _: P, _: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    unreachable!()
  }
}

pub struct RenderableError<const NODE_NAME_SIZE: usize, E: FileforgeError, P: DiagnosticPoolProvider> {
  error: E,
  render_mode: RenderMode,
  provider: P,
}

impl<const NODE_NAME_SIZE: usize, E: FileforgeError, P: DiagnosticPoolProvider> RenderableError<NODE_NAME_SIZE, E, P> {
  pub fn from_error(error: E, render_mode: RenderMode, provider: P) -> Self {
    Self { error, render_mode, provider }
  }
}

pub trait RenderableResult<'pool, T, P: DiagnosticPoolProvider> {
  fn unwrap_renderable<const NODE_NAME_SIZE: usize>(self, render_mode: RenderMode, provider: &'pool P) -> T;
}

impl<'pool, P: DiagnosticPoolProvider + 'pool, T, E: FileforgeError> RenderableResult<'pool, T, P> for Result<T, E> {
  fn unwrap_renderable<const NODE_NAME_SIZE: usize>(self, render_mode: RenderMode, provider: &'pool P) -> T {
    self.map_err(|error| RenderableError::<NODE_NAME_SIZE, E, _>::from_error(error, render_mode, provider)).unwrap()
  }
}

impl<'pool, const NODE_NAME_SIZE: usize, E: FileforgeError, P: DiagnosticPoolProvider + 'pool> core::fmt::Debug for RenderableError<NODE_NAME_SIZE, E, &'pool P> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.error.render_into_report(self.provider, |report: Report<'_, '_, NODE_NAME_SIZE, &'pool P>| {
      let mut o = 0;

      loop {
        let mut buffer = [RenderBufferCell::default(); 80];
        let mut buffer = RenderBuffer::new(&mut buffer, 80, o);
        o += 1;

        let r = buffer.canvas_at(RenderPosition::zero()).write(&report).unwrap();

        if r.get_line_height() <= o {
          break;
        }

        let _ = buffer.flush_into(f, self.render_mode);
      }
    });

    Ok(())
  }
}
