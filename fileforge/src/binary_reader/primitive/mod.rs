pub mod bool;
pub mod numeric;
pub mod slice;
pub mod unit;

use super::endianness::Endianness;

pub trait Primitive<const SIZE: usize> {
  fn read(data: &[u8; SIZE], endianness: Endianness) -> Self;
  fn write(&self, data: &mut [u8; SIZE], endianness: Endianness);
}
