#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::error::render::{buffer::canvas::RenderBufferCanvas, r#trait::renderable::Renderable};

use super::number::formatted_unsigned::FormattedUnsigned;

pub struct ByteDisplay<const NO_ALLOC_SIZE: usize> {
  #[cfg(feature = "alloc")]
  data: Vec<u8>,

  #[cfg(not(feature = "alloc"))]
  data: [u8; NO_ALLOC_SIZE],

  #[cfg(not(feature = "alloc"))]
  len: u64,
}

impl<const NO_ALLOC_SIZE: usize> ByteDisplay<NO_ALLOC_SIZE> {
  pub fn len(&self) -> u64 {
    #[cfg(feature = "alloc")]
    return self.data.len() as u64;

    #[cfg(not(feature = "alloc"))]
    return self.len;
  }

  pub fn new(data: &[u8]) -> ByteDisplay<NO_ALLOC_SIZE> {
    #[cfg(feature = "alloc")]
    {
      ByteDisplay { data: Vec::from(data) }
    }

    #[cfg(not(feature = "alloc"))]
    {
      let mut stack_data: [u8; NO_ALLOC_SIZE] = [0; NO_ALLOC_SIZE];

      stack_data[0..min(NO_ALLOC_SIZE, data.len())].copy_from_slice(&data[0..min(NO_ALLOC_SIZE, data.len())]);

      ByteDisplay {
        data: stack_data,
        len: data.len() as u64,
      }
    }
  }
}

impl<'t, const NO_ALLOC_SIZE: usize> Renderable<'t> for ByteDisplay<NO_ALLOC_SIZE> {
  fn render_into<'r, 'c>(&self, canvas: &mut RenderBufferCanvas<'r, 'c, 't>) -> Result<(), ()> {
    let max_len = FormattedUnsigned::new(self.len() as u128).length();

    for (byte_row, row) in self.data.chunks(16).zip(0..) {
      canvas.write(&FormattedUnsigned::new(row as u128).base(16).padding(max_len))?;
      canvas.set_str(" | ");

      for (byte, col) in byte_row.iter().zip(0..) {
        canvas.write(&FormattedUnsigned::new(*byte as u128).base(16).padding(2))?;

        if col != 15 {
          canvas.set_char(" ");
        }
      }

      canvas.cursor_down();
      canvas.set_column(canvas.get_start_position().column());
    }

    Ok(())
  }
}
