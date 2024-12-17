#![no_std]

pub mod diagnostic;
pub mod error;
pub mod provider;
pub mod reader;

#[cfg(feature = "alloc")]
extern crate alloc;
