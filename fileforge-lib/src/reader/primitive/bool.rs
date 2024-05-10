use crate::reader::r#trait::primitive::Primitive;

pub struct Bool8(pub bool);
impl Primitive<1> for Bool8 {
  fn read_be(data: &[u8; 1]) -> Self { Bool8(u8::from_be_bytes(*data) != 0) }
  fn read_le(data: &[u8; 1]) -> Self { Bool8(u8::from_le_bytes(*data) != 0) }
}

pub struct Bool16(pub bool);
impl Primitive<2> for Bool16 {
  fn read_be(data: &[u8; 2]) -> Self { Bool16(u16::from_be_bytes(*data) != 0) }
  fn read_le(data: &[u8; 2]) -> Self { Bool16(u16::from_le_bytes(*data) != 0) }
}

pub struct Bool32(pub bool);
impl Primitive<4> for Bool32 {
  fn read_be(data: &[u8; 4]) -> Self { Bool32(u32::from_be_bytes(*data) != 0) }
  fn read_le(data: &[u8; 4]) -> Self { Bool32(u32::from_le_bytes(*data) != 0) }
}

pub struct Bool64(pub bool);
impl Primitive<8> for Bool64 {
  fn read_be(data: &[u8; 8]) -> Self { Bool64(u64::from_be_bytes(*data) != 0) }
  fn read_le(data: &[u8; 8]) -> Self { Bool64(u64::from_le_bytes(*data) != 0) }
}

pub struct Bool128(pub bool);
impl Primitive<16> for Bool128 {
  fn read_be(data: &[u8; 16]) -> Self { Bool128(u128::from_be_bytes(*data) != 0) }
  fn read_le(data: &[u8; 16]) -> Self { Bool128(u128::from_le_bytes(*data) != 0) }
}