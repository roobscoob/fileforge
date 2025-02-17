pub mod renderable;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Magic<const SIZE: usize> {
  bytes: [u8; SIZE],
}

impl<const SIZE: usize> Magic<SIZE> {
  pub const fn from_bytes(bytes: [u8; SIZE]) -> Magic<SIZE> { Self { bytes } }
  pub const fn from_byte_ref(bytes: &[u8; SIZE]) -> Magic<SIZE> { Self { bytes: *bytes } }
}
