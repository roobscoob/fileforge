use core::marker::PhantomData;

use render::{buffer::{cell::{tag::context::RenderMode, RenderBufferCell}, RenderBuffer}, position::RenderPosition};
use report::Report;

pub mod context;
pub mod render;
pub mod report;

pub trait FileforgeError<'pool, const NODE_NAME_SIZE: usize> {
  fn render_into_report(&self, callback: impl for<'a, 'b> FnMut(Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ());
}

impl<'pool, const NODE_NAME_SIZE: usize> FileforgeError<'pool, NODE_NAME_SIZE> for core::convert::Infallible {
  fn render_into_report(&self, _: impl for<'a, 'b> FnMut(Report<'a, 'b, 'pool, NODE_NAME_SIZE>) -> ()) { unreachable!() }
}

pub struct RenderableError<'pool, const NODE_NAME_SIZE: usize, E: FileforgeError<'pool, NODE_NAME_SIZE>> {
  error: E,
  render_mode: RenderMode,
  ph: PhantomData<&'pool ()>
}

impl<'pool, const NODE_NAME_SIZE: usize, E: FileforgeError<'pool, NODE_NAME_SIZE>> From<E> for RenderableError<'pool, NODE_NAME_SIZE, E> {
  fn from(error: E) -> Self {
    RenderableError { error, render_mode: RenderMode::TerminalAnsi, ph: PhantomData::default() }
  }
}

pub trait RenderableResult<'pool, const NODE_NAME_SIZE: usize, T> {
  fn unwrap_renderable(self) -> T;
}

impl<'pool, const NODE_NAME_SIZE: usize, T, E: FileforgeError<'pool, NODE_NAME_SIZE>> RenderableResult<'pool, NODE_NAME_SIZE, T> for Result<T, E> {
  fn unwrap_renderable(self) -> T {
    self
      .map_err(|e| Into::<RenderableError<NODE_NAME_SIZE, E>>::into(e))
      .unwrap()
  }
}

impl<'pool, const NODE_NAME_SIZE: usize, T: FileforgeError<'pool, NODE_NAME_SIZE>> core::fmt::Debug for RenderableError<'pool, NODE_NAME_SIZE, T> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    self.error.render_into_report(|report| {
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