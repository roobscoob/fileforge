pub mod error;

use error::BymlHeaderError;

use crate::{diagnostic::node::{name::DiagnosticNodeName, reference::DiagnosticReference, tagged_reference::TaggedDiagnosticReference}, provider::r#trait::Provider, reader::{self, error::ParseError, r#trait::readable::FixedSizeReadable, Reader}, object::endianness::EndiannessMarker};

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

  pub fn version_diagnostic(&self) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference.create_physical_child(2, 2, DiagnosticNodeName::from("Version"))
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, 0xA> for BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
  type Argument = ();
  type Error = BymlHeaderError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<RP: Provider>(reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>, argument: Self::Argument) -> Result<Self, ParseError<'pool, Self::Error, RP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let dr = reader.diagnostic_reference();
    
    let endianness_marker: EndiannessMarker<2> = reader.read_with(DiagnosticNodeName::from("Endianness"), EndiannessMarker::big(*b"BY"))
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