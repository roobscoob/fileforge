pub mod error;

use error::BymlHeaderError;

use fileforge_lib::{
  diagnostic::node::{name::DiagnosticNodeName, reference::DiagnosticReference},
  provider::r#trait::Provider,
  reader::{self, error::parse::ParseError, r#trait::readable::FixedSizeReadable, Reader},
};
use fileforge_std::endianness::EndiannessMarker;

pub struct BymlHeader<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  diagnostic_reference: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  pub endianness: reader::endianness::Endianness,
  pub version: u16,
  pub key_table_offset: u32,
  pub string_table_offset: u32,
  pub root_data_offset: u32,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
  pub fn size() -> u64 { 16 }

  pub fn endianness_diagnostic(&self) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference.create_physical_child(
      0,
      Some(2),
      DiagnosticNodeName::from("Endianness"),
    )
  }

  pub fn version_diagnostic(&self) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self
      .diagnostic_reference
      .create_physical_child(2, Some(2), DiagnosticNodeName::from("Version"))
  }

  pub fn key_table_offset_diagnostic(
    &self,
  ) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference.create_physical_child(
      4,
      Some(4),
      DiagnosticNodeName::from("Key Table Offset"),
    )
  }

  pub fn string_table_offset_diagnostic(
    &self,
  ) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference.create_physical_child(
      8,
      Some(4),
      DiagnosticNodeName::from("String Table Offset"),
    )
  }

  pub fn root_data_offset_diagnostic(
    &self,
  ) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference.create_physical_child(
      12,
      Some(4),
      DiagnosticNodeName::from("Root Node Offset"),
    )
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, 0x10>
  for BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE>
{
  type Argument = ();
  type Error = BymlHeaderError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<RP: Provider>(
    reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
    _argument: Self::Argument,
  ) -> Result<
    Self,
    ParseError<'pool, Self::Error, RP::ReadError, RP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    let dr = reader.diagnostic_reference();

    let endianness_marker: EndiannessMarker<2> = reader
      .read_with(
        DiagnosticNodeName::from("Endianness"),
        EndiannessMarker::big(*b"BY"),
      )
      .map_err(|e| e.map_domains(BymlHeaderError::Endianness))?;

    let endianness = endianness_marker.endianness();
    reader.set_endianness(endianness);

    Ok(BymlHeader {
      diagnostic_reference: dr,
      endianness,
      version: reader.get("Version")?,
      key_table_offset: reader.get("Key Table Offset")?,
      string_table_offset: reader.get("String Table Offset")?,
      root_data_offset: reader.get("Root Data Offset")?,
    })
  }
}
