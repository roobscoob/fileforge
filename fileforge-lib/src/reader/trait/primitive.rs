pub trait Primitive<const SIZE: usize>: Sized {
  fn read_le(data: &[u8; SIZE]) -> Self;
  fn read_be(data: &[u8; SIZE]) -> Self;
}
