use core::convert::Infallible;

use fileforge::{
  binary_reader::{primitive::Primitive, readable::Readable, BinaryReader, PrimitiveReader},
  stream::ReadableStream,
};

pub mod renderable;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Version<const SEGMENTS: usize> {
  versions: [u8; SEGMENTS],
}

impl<const SEGMENTS: usize> Version<SEGMENTS> {
  pub fn new_raw(contents: [u8; SEGMENTS]) -> Self {
    Self { versions: contents }
  }
}

impl Version<1> {
  pub fn new(version: u8) -> Self {
    Self { versions: [version] }
  }
}

impl Version<2> {
  pub fn new(major: u8, minor: u8) -> Self {
    Self { versions: [major, minor] }
  }
}

impl Version<3> {
  pub fn new(major: u8, minor: u8, patch: u8) -> Self {
    Self { versions: [major, minor, patch] }
  }
}

impl Version<4> {
  pub fn new(major: u8, minor: u8, patch: u8, revision: u8) -> Self {
    Self {
      versions: [major, minor, patch, revision],
    }
  }
}

impl<'pool: 'l, 'l, const SEGMENTS: usize> Primitive<SEGMENTS> for Version<SEGMENTS> {
  fn read(data: &[u8; SEGMENTS], endianness: fileforge::binary_reader::endianness::Endianness) -> Self {
    Self::new_raw(*data)
  }

  fn write(&self, data: &mut [u8; SEGMENTS], endianness: fileforge::binary_reader::endianness::Endianness) {
    *data = self.versions;
  }
}
