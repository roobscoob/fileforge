use crate::reader::r#trait::primitive::Primitive;

impl Primitive<1> for u8 {
  fn read_be(data: &[u8; 1]) -> Self { u8::from_be_bytes(*data) }
  fn read_le(data: &[u8; 1]) -> Self { u8::from_le_bytes(*data) }
}

impl Primitive<1> for i8 {
  fn read_be(data: &[u8; 1]) -> Self { i8::from_be_bytes(*data) }
  fn read_le(data: &[u8; 1]) -> Self { i8::from_le_bytes(*data) }
}

impl Primitive<2> for u16 {
  fn read_be(data: &[u8; 2]) -> Self { u16::from_be_bytes(*data) }
  fn read_le(data: &[u8; 2]) -> Self { u16::from_le_bytes(*data) }
}

impl Primitive<2> for i16 {
  fn read_be(data: &[u8; 2]) -> Self { i16::from_be_bytes(*data) }
  fn read_le(data: &[u8; 2]) -> Self { i16::from_le_bytes(*data) }
}

impl Primitive<3> for intx::U24 {
  fn read_be(data: &[u8; 3]) -> Self { intx::U24::from_be_bytes(*data) }
  fn read_le(data: &[u8; 3]) -> Self { intx::U24::from_le_bytes(*data) }
}

impl Primitive<3> for intx::I24 {
  fn read_be(data: &[u8; 3]) -> Self { intx::I24::from_be_bytes(*data) }
  fn read_le(data: &[u8; 3]) -> Self { intx::I24::from_le_bytes(*data) }
}

impl Primitive<4> for u32 {
  fn read_be(data: &[u8; 4]) -> Self { u32::from_be_bytes(*data) }
  fn read_le(data: &[u8; 4]) -> Self { u32::from_le_bytes(*data) }
}

impl Primitive<4> for i32 {
  fn read_be(data: &[u8; 4]) -> Self { i32::from_be_bytes(*data) }
  fn read_le(data: &[u8; 4]) -> Self { i32::from_le_bytes(*data) }
}

impl Primitive<5> for intx::U40 {
  fn read_be(data: &[u8; 5]) -> Self { intx::U40::from_be_bytes(*data) }
  fn read_le(data: &[u8; 5]) -> Self { intx::U40::from_le_bytes(*data) }
}

impl Primitive<5> for intx::I40 {
  fn read_be(data: &[u8; 5]) -> Self { intx::I40::from_be_bytes(*data) }
  fn read_le(data: &[u8; 5]) -> Self { intx::I40::from_le_bytes(*data) }
}

impl Primitive<6> for intx::U48 {
  fn read_be(data: &[u8; 6]) -> Self { intx::U48::from_be_bytes(*data) }
  fn read_le(data: &[u8; 6]) -> Self { intx::U48::from_le_bytes(*data) }
}

impl Primitive<6> for intx::I48 {
  fn read_be(data: &[u8; 6]) -> Self { intx::I48::from_be_bytes(*data) }
  fn read_le(data: &[u8; 6]) -> Self { intx::I48::from_le_bytes(*data) }
}

impl Primitive<7> for intx::U56 {
  fn read_be(data: &[u8; 7]) -> Self { intx::U56::from_be_bytes(*data) }
  fn read_le(data: &[u8; 7]) -> Self { intx::U56::from_le_bytes(*data) }
}

impl Primitive<7> for intx::I56 {
  fn read_be(data: &[u8; 7]) -> Self { intx::I56::from_be_bytes(*data) }
  fn read_le(data: &[u8; 7]) -> Self { intx::I56::from_le_bytes(*data) }
}

impl Primitive<8> for u64 {
  fn read_be(data: &[u8; 8]) -> Self { u64::from_be_bytes(*data) }
  fn read_le(data: &[u8; 8]) -> Self { u64::from_le_bytes(*data) }
}

impl Primitive<8> for i64 {
  fn read_be(data: &[u8; 8]) -> Self { i64::from_be_bytes(*data) }
  fn read_le(data: &[u8; 8]) -> Self { i64::from_le_bytes(*data) }
}

impl Primitive<16> for u128 {
  fn read_be(data: &[u8; 16]) -> Self { u128::from_be_bytes(*data) }
  fn read_le(data: &[u8; 16]) -> Self { u128::from_le_bytes(*data) }
}

impl Primitive<16> for i128 {
  fn read_be(data: &[u8; 16]) -> Self { i128::from_be_bytes(*data) }
  fn read_le(data: &[u8; 16]) -> Self { i128::from_le_bytes(*data) }
}

impl Primitive<4> for f32 {
  fn read_be(data: &[u8; 4]) -> Self { f32::from_be_bytes(*data) }
  fn read_le(data: &[u8; 4]) -> Self { f32::from_le_bytes(*data) }
}

impl Primitive<8> for f64 {
  fn read_be(data: &[u8; 8]) -> Self { f64::from_be_bytes(*data) }
  fn read_le(data: &[u8; 8]) -> Self { f64::from_le_bytes(*data) }
}