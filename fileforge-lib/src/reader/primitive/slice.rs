use crate::reader::r#trait::primitive::Primitive;

impl<const SIZE: usize> Primitive<SIZE> for [u8; SIZE] {
  fn read_be(data: &[u8; SIZE]) -> Self { *data }
  fn read_le(data: &[u8; SIZE]) -> Self { *data }
}

pub struct FixedCString<const SIZE: usize> {
  contents: [u8; SIZE],
}

impl<const SIZE: usize> TryFrom<&str> for FixedCString<SIZE> {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    let bytes = value.as_bytes();

    if bytes.len() > SIZE {
      return Err(());
    }

    let mut contents = [0; SIZE];

    contents[0..bytes.len()].copy_from_slice(bytes);

    Ok(FixedCString { contents })
  }
}

impl<const SIZE: usize> Primitive<SIZE> for FixedCString<SIZE> {
  fn read_be(data: &[u8; SIZE]) -> Self { Self { contents: *data } }
  fn read_le(data: &[u8; SIZE]) -> Self { Self { contents: *data } }
}
