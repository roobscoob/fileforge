use core::fmt::Write;

use super::{
  buffer::{
    cell::{tag::context::RenderMode, RenderBufferCell},
    RenderBuffer,
  },
  position::RenderPosition,
  r#trait::renderable::Renderable,
};

pub struct RenderSession {}

impl RenderSession {
  pub fn render_to_writable<'t, R: Renderable<'t>>(
    renderable: R,
    buffer: &'t mut [RenderBufferCell<'t>],
    into: &mut dyn Write,
    mode: RenderMode,
  ) -> core::fmt::Result {
    let mut offset = 0;

    loop {
      let mut buf = RenderBuffer::new(buffer, buffer.len(), offset);

      let mut c = buf.canvas_at(RenderPosition::zero());
      renderable.render_into(&mut c).unwrap();

      if buf.is_empty() {
        break Ok(());
      }

      buf.flush_into(into, mode)?;

      for cell in buffer.iter_mut() {
        cell.clear();
      }

      offset += 1;
    }
  }
}
