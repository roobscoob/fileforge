// #![no_std]

pub mod diagnostic;
pub mod object;
pub mod provider;
pub mod reader;
pub mod error;

#[cfg(feature = "alloc")]
extern crate alloc;