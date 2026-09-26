pub mod mutable;
pub mod readable;

pub static YAZ0_HEADER_SIZE: usize = 0x10;

pub struct Yaz0Header {
  decompressed_size: u32,
  data_alignment: u32,

  #[allow(unused)]
  unused: u32,
}

impl Yaz0Header {
  pub fn empty() -> Self {
    Self {
      decompressed_size: 0,
      data_alignment: 0,
      unused: 0,
    }
  }

  pub fn with_decompressed_size(self, size: u32) -> Self {
    Self { decompressed_size: size, ..self }
  }

  pub fn with_alignment(self, alignment: u32) -> Self {
    Self { data_alignment: alignment, ..self }
  }

  pub fn decompressed_size(&self) -> u32 {
    self.decompressed_size
  }
  pub fn alignment(&self) -> u32 {
    self.data_alignment
  }
}

#[cfg(test)]
mod tests {
  use fileforge::{
    binary_reader::{endianness::Endianness, view::View, BinaryReader},
    provider::hint::ReadHint,
  };

  use super::Yaz0Header;

  #[tokio::test]
  async fn a_view_reports_the_values_it_was_changed_to() {
    let mut bytes = [&b"Yaz0"[..], &100u32.to_be_bytes(), &0u32.to_be_bytes(), &[0; 4]].concat();

    let reader = BinaryReader::new_from_provider(&mut bytes, Endianness::BigEndian, ReadHint::new());
    let mut view = reader.into::<View<'_, _, Yaz0Header>>().await.ok().unwrap();
    assert_eq!(view.decompressed_size(), 100);

    let mutator = view.mutate().await.ok().unwrap();
    mutator.with_uncompressed_size(250).await.ok().unwrap().with_alignment(16).await.ok().unwrap();

    assert_eq!(view.decompressed_size(), 250);
    assert_eq!(view.alignment(), 16);

    drop(view);
    assert_eq!(&bytes[4..12], &[0, 0, 0, 250, 0, 0, 0, 16]);
  }
}
