pub mod error;

use error::BymlHeaderError;

use crate::{diagnostic::node::name::DiagnosticNodeName, provider::r#trait::Provider, reader::{self, endianness::Endianness, error::ParseError, r#trait::readable::FixedSizeReadable, Reader}, object::endianness::EndiannessMarker};

pub struct BymlHeader {
  endianness: reader::endianness::Endianness,
  version: u16,
  key_table_offset: u16,
  string_table_offset: u16,
  root_data_offset: u16,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, 0xA> for BymlHeader {
  type Argument = ();
  type Error = BymlHeaderError<'pool, DIAGNOSTIC_NODE_NAME_SIZE>;

  fn read<RP: Provider>(reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>, argument: Self::Argument) -> Result<Self, ParseError<'pool, Self::Error, RP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let endianness_marker: EndiannessMarker<2> = reader.read_with(DiagnosticNodeName::from("Endianness"), EndiannessMarker::big(*b"BY"))
      .map_err(|e| e.map_domains(BymlHeaderError::Endianness))?;

    let endianness = endianness_marker.endianness();
    reader.set_endianness(endianness);

    Ok(BymlHeader {
      endianness,
      version: reader.get("Version")?,
      key_table_offset: reader.get("Key Table Offset")?,
      string_table_offset: reader.get("String Table Offset")?,
      root_data_offset: reader.get("Root Data Offset")?,
    })
  }
}