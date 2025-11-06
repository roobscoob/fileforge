use crate::error::render::{
  buffer::{
    RenderBuffer,
    cell::{RenderBufferCell, tag::context::RenderMode},
  },
  position::RenderPosition,
};

use imports::*;

#[doc(hidden)]
pub(crate) mod imports {
  pub use crate::{
    diagnostic::pool::fixed::{FixedDiagnosticPool, entry::FixedDiagnosticPoolEntry},
    error::report::Report,
  };
}

#[doc(hidden)]
pub const NODE_NAME_SIZE: usize = 128;
pub struct Story {
  pub name: &'static str,
  pub type_name: &'static str,
  pub story: fn(FixedDiagnosticPool<'_, NODE_NAME_SIZE>, fn(Report<NODE_NAME_SIZE, FixedDiagnosticPool<'_, NODE_NAME_SIZE>>)),
}

pub fn invoke() {
  for ele in inventory::iter::<Story> {
    let mut entries: [FixedDiagnosticPoolEntry<NODE_NAME_SIZE>; 256] = core::array::from_fn(|_| FixedDiagnosticPoolEntry::default());
    let pool = FixedDiagnosticPool::new(&mut entries);
    println!("{} ({})", ele.name, ele.type_name);
    (ele.story)(pool, |report| {
      let mut o = 0;

      loop {
        let mut buffer = [RenderBufferCell::default(); 80];
        let mut buffer = RenderBuffer::new(&mut buffer, 80, o);
        o += 1;

        let r = buffer.canvas_at(RenderPosition::zero()).write(&report).unwrap();

        if r.get_line_height() <= o {
          break;
        }

        let mut s = String::new();

        buffer.flush_into(&mut s, RenderMode::TerminalAnsi).unwrap();

        print!("{}", s);
      }
    });
    println!();
  }
}

pub fn iter_stories() -> impl Iterator<Item = &'static Story> {
  inventory::iter::<Story>.into_iter()
}

inventory::collect!(Story);
