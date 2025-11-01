pub mod mutable;
pub mod readable;

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
