use core::num::NonZero;

use fileforge::binary_reader::endianness::Endianness;

pub mod readable;

pub static BYML_HEADER_SIZE: usize = 0x10;

#[derive(Debug)]
pub struct BymlHeader {
  config: BymlHeaderConfig,
  endianness: Endianness,
  version: u16,
  key_table_offset: u32,
  string_table_offset: u32,
  binary_data_table_offset: u32,
  root_data_offset: u32,
}

impl BymlHeader {
  pub fn size(&self) -> u64 {
    if self.config.feat_binary_data_table {
      0x14
    } else {
      0x10
    }
  }

  pub fn config(&self) -> BymlHeaderConfig {
    self.config
  }

  pub fn version(&self) -> u16 {
    self.version
  }

  pub fn key_table_offset(&self) -> Option<NonZero<u32>> {
    NonZero::new(self.key_table_offset)
  }

  pub fn string_table_offset(&self) -> Option<NonZero<u32>> {
    NonZero::new(self.string_table_offset)
  }

  pub fn binary_data_table_offset(&self) -> Option<NonZero<u32>> {
    NonZero::new(self.binary_data_table_offset)
  }

  pub fn root_data_offset(&self) -> Option<NonZero<u32>> {
    NonZero::new(self.root_data_offset)
  }
}

#[derive(Clone, Copy, Debug)]
pub struct BymlHeaderConfig {
  pub feat_binary_data_table: bool,
}

impl BymlHeaderConfig {
  pub fn build() -> BymlHeaderConfigBuilder {
    BymlHeaderConfigBuilder::new()
  }
}

pub struct BymlHeaderConfigBuilder {
  include_binary_data_table: bool,
}

impl BymlHeaderConfigBuilder {
  /// Creates a new builder with default values
  pub fn new() -> Self {
    Self { include_binary_data_table: false }
  }

  /// Sets whether to include a binary data table
  pub fn include_binary_data_table(mut self, include: bool) -> Self {
    self.include_binary_data_table = include;
    self
  }

  /// Sets the binary data table to be included
  pub fn with_binary_data_table(mut self) -> Self {
    self.include_binary_data_table = true;
    self
  }

  /// Sets the binary data table to be excluded
  pub fn without_binary_data_table(mut self) -> Self {
    self.include_binary_data_table = false;
    self
  }

  /// Builds the final BymlHeaderConfig
  pub fn build(self) -> BymlHeaderConfig {
    BymlHeaderConfig {
      feat_binary_data_table: self.include_binary_data_table,
    }
  }
}

impl Default for BymlHeaderConfigBuilder {
  fn default() -> Self {
    Self::new()
  }
}

// Optional: Add Default implementation for BymlHeaderConfig
impl Default for BymlHeaderConfig {
  fn default() -> Self {
    BymlHeaderConfigBuilder::new().build()
  }
}
