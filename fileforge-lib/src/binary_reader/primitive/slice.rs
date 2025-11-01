use super::Primitive;

impl<const T: usize> Primitive<T> for [u8; T] {
  fn read(data: &[u8; T], _: crate::binary_reader::endianness::Endianness) -> Self {
    *data
  }

  fn write(&self, data: &mut [u8; T], _: crate::binary_reader::endianness::Endianness) {
    *data = *self
  }
}
