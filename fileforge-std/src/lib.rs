#![no_std]

pub mod endianness;
pub mod magic;
pub mod providers;

#[cfg(feature = "std")]
extern crate std;
