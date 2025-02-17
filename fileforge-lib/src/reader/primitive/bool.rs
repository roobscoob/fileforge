use crate::reader::endianness::Endianness;

use super::Primitive;

impl Primitive<1> for bool {
  fn read(data: &[u8; 1], _: Endianness) -> Self { data[0] != 0 }
  fn write(&self, data: &mut [u8; 1], _: Endianness) { data[0] = if *self { 1 } else { 0 } }
}
