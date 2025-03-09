use render::{buffer::{cell::{tag::context::RenderMode, RenderBufferCell}, RenderBuffer}, position::RenderPosition};
use report::Report;

use crate::diagnostic::pool::DiagnosticPoolProvider;

pub mod context;
pub mod render;
pub mod report;

pub trait FileforgeError {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(&self, provider: &'pool_ref P, callback: impl for<'tag, 'b, 'pool> FnMut(Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> ());
}

impl FileforgeError for core::convert::Infallible {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(&self, _: &'pool_ref P, _: impl for<'tag, 'b, 'pool> FnMut(Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> ()) { unreachable!() }
}

pub struct RenderableError<'pool_ref, const NODE_NAME_SIZE: usize, E: FileforgeError, P: DiagnosticPoolProvider> {
  error: E,
  render_mode: RenderMode,
  provider: &'pool_ref P,
}

impl<'pool_ref, const NODE_NAME_SIZE: usize, E: FileforgeError, P: DiagnosticPoolProvider> RenderableError<'pool_ref, NODE_NAME_SIZE, E, P> {
  pub fn from_error(error: E, render_mode: RenderMode, provider: &'pool_ref P) -> Self {
    Self { error, render_mode, provider }
  }
}

pub trait RenderableResult<T> {
  fn unwrap_renderable<'pool_ref, const NODE_NAME_SIZE: usize>(self, render_mode: RenderMode, provider: &'pool_ref impl DiagnosticPoolProvider) -> T;
}

impl<T, E: FileforgeError> RenderableResult<T> for Result<T, E> {
  fn unwrap_renderable<'pool_ref, const NODE_NAME_SIZE: usize>(self, render_mode: RenderMode, provider: &'pool_ref impl DiagnosticPoolProvider) -> T {
    self
      .map_err(|error| RenderableError::<NODE_NAME_SIZE, E, _>::from_error(error, render_mode, provider))
      .unwrap()
  }
}

impl<'pool_ref, const NODE_NAME_SIZE: usize, E: FileforgeError, P: DiagnosticPoolProvider> core::fmt::Debug for RenderableError<'pool_ref, NODE_NAME_SIZE, E, P> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.error.render_into_report(self.provider, |report: Report<'_, '_, '_, '_, NODE_NAME_SIZE, P>| {
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