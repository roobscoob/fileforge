use error::{get_root_node::GetRootNodeError, get_string_table::{GetStringTableError, StringTableOutOfBounds}};
use string_table::{error::size::StringTableSizeError, StringTable};

use fileforge_lib::{diagnostic::{node::name::DiagnosticNodeName, pool::DiagnosticPool}, provider::{slice::dynamic::DynamicSliceProvider, r#trait::Provider}, reader::{endianness::Endianness, error::{parse::ParseErrorResultExtension, parse_primitive::ParsePrimitiveErrorResultExtension}, Reader, SeekFrom}};

use self::{header::BymlHeader, error::{load::LoadError, get_header::GetHeaderError}};

pub mod header;
pub mod string_table;
pub mod error;
pub mod node;

pub struct BymlReader<'provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  provider: &'provider P,
  pool: &'pool &'pool mut DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  endianness: Option<Endianness>,
}

impl<'provider, 'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> BymlReader<'provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, P> {
  fn compute_header_size() -> u64 { BymlHeader::<'static, 0>::size() }

  fn header(&self) -> Result<BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE>, GetHeaderError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let mut reader = Reader::<'pool, DIAGNOSTIC_NODE_NAME_SIZE, &P>::root(&*self.provider, self.endianness.unwrap_or(Endianness::Big), *self.pool, DiagnosticNodeName::from("Byml"));

    let header: BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE> = reader.read(DiagnosticNodeName::from("Header"))
      .map_domain_specific(|v| { GetHeaderError::HeaderError(v) })?
      .map_out_of_bounds(|oob| { GetHeaderError::MissingData(oob) })?
      .map_err(|provider_read| { GetHeaderError::ProviderError(provider_read) })?;

    Ok(header)
  }

  pub fn construct_in() {}

  pub fn version(&self) -> Result<u16, GetHeaderError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.header()?.version)
  }

  pub fn endianness(&self) -> Result<Endianness, GetHeaderError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.header()?.endianness)
  }

  pub fn over(provider: &'provider P, pool: &'pool &'pool mut DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<BymlReader<'provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>, LoadError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let mut unmanaged_byml = BymlReader { provider, pool, endianness: None };
    let header = unmanaged_byml.header()
      .map_err(|e| LoadError::HeaderGetError(e))?;

    unmanaged_byml.endianness = Some(header.endianness);

    LoadError::assert_supported(header.version, header.endianness, || header.version_diagnostic())?;

    Ok(unmanaged_byml)
  }

  pub fn string_table(&self) -> Result<StringTable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, DynamicSliceProvider<'_, <P as Provider>::DynReturnedProviderType>>, GetStringTableError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let header = self.header()
      .map_err(|e| GetStringTableError::GetHeaderError(e))?;

    let mut reader = Reader::<'pool, DIAGNOSTIC_NODE_NAME_SIZE, &P>::root(&*self.provider, self.endianness.unwrap_or(Endianness::Big), *self.pool, DiagnosticNodeName::from("Byml"));

    reader.seek(SeekFrom::Start(header.string_table_offset as u64))
      .map_err(|_| GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
        string_table_parent: reader.diagnostic_reference(),
        string_table_size: None,
        string_table_size_complete: false,
        string_table_position: header.string_table_offset as usize,
        string_table_position_dr: header.string_table_offset_diagnostic(),
        string_table_size_dr: None,
        byml_size: self.provider.len() as usize,
      }))?;

    let (size, drb) = StringTable::size(&mut reader, self.provider.len() as usize, || header.string_table_offset_diagnostic())
      .map_err(|e| GetStringTableError::GetStringTableSizeError(e))?;

    let dr = reader.diagnostic_reference().create_physical_child(header.string_table_offset as u64, size, DiagnosticNodeName::from("String Table"));

    let provider = self.provider.slice_dyn(header.string_table_offset as u64, size)
      .map_err(|_| GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
        string_table_parent: reader.diagnostic_reference(),
        string_table_position: header.string_table_offset as usize,
        string_table_position_dr: header.string_table_offset_diagnostic(),
        string_table_size: Some(size as usize),
        string_table_size_complete: true,
        string_table_size_dr: Some(drb),
        byml_size: self.provider.len() as usize,
      }))?;

    let reader = Reader::at(provider, header.endianness, dr);

    Ok(StringTable { reader })
  }

  pub fn key_table(&self) -> Result<StringTable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, DynamicSliceProvider<'_, <P as Provider>::DynReturnedProviderType>>, GetStringTableError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let header = self.header()
      .map_err(|e| GetStringTableError::GetHeaderError(e))?;

    let mut reader = Reader::<'pool, DIAGNOSTIC_NODE_NAME_SIZE, &P>::root(&*self.provider, self.endianness.unwrap_or(Endianness::Big), *self.pool, DiagnosticNodeName::from("Byml"));

    reader.seek(SeekFrom::Start(header.key_table_offset as u64))
      .map_err(|_| GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
        string_table_parent: reader.diagnostic_reference(),
        string_table_size: None,
        string_table_size_complete: false,
        string_table_position: header.key_table_offset as usize,
        string_table_position_dr: header.key_table_offset_diagnostic(),
        string_table_size_dr: None,
        byml_size: self.provider.len() as usize,
      }))?;

    let (size, drb) = StringTable::size(&mut reader, self.provider.len() as usize, || header.key_table_offset_diagnostic())
      .map_err(|e| GetStringTableError::GetStringTableSizeError(e))?;

    let dr = reader.diagnostic_reference().create_physical_child(header.key_table_offset as u64, size, DiagnosticNodeName::from("String Table"));

    let provider = self.provider.slice_dyn(header.key_table_offset as u64, size)
      .map_err(|_| GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
        string_table_parent: reader.diagnostic_reference(),
        string_table_position: header.key_table_offset as usize,
        string_table_position_dr: header.key_table_offset_diagnostic(),
        string_table_size: Some(size as usize),
        string_table_size_complete: true,
        string_table_size_dr: Some(drb),
        byml_size: self.provider.len() as usize,
      }))?;

    let reader = Reader::at(provider, header.endianness, dr);

    Ok(StringTable { reader })
  }

  pub fn root(&self) -> Result<(), GetRootNodeError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let header = self.header()
      .map_err(|e| GetRootNodeError::GetHeaderError(e))?;

    header.root_data_offset;

    Ok(())
  }
}