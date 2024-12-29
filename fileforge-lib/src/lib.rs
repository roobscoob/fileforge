#![no_std]

macro_rules! const_text {
  ($x: tt $y: tt) => {{
    const R: [&'static dyn $crate::error::render::buffer::cell::tag::CellTag; 1] = $x;
    const T: $crate::error::render::builtin::text::r#const::ConstText =
      $crate::error::render::builtin::text::r#const::ConstText::new($y, R[0]);

    &T
  }};
}

pub mod diagnostic;
pub mod error;
pub mod provider;
pub mod reader;
pub mod stream;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;
