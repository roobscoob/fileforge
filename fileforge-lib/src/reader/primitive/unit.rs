use super::Primitive;

impl Primitive<0> for () {
  fn read(_: &[u8; 0], _: crate::reader::endianness::Endianness) -> Self {}
  fn write(&self, _: &mut [u8; 0], _: crate::reader::endianness::Endianness) {}
}
