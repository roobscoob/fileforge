#![no_std]

pub mod unmanaged;
pub mod util;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod managed;
