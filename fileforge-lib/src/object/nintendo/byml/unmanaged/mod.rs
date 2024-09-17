use crate::{provider::r#trait::Provider, reader::{Reader, endianness::Endianness, error::{ParseErrorResultExtension, ParsePrimitiveErrorResultExtension}, SeekFrom}, diagnostic::{node::name::DiagnosticNodeName, pool::DiagnosticPool}};

use self::{header::BymlHeader, error::{load::LoadError, get_header::GetHeaderError}};

pub mod header;
pub mod string_table;
pub mod error;

pub struct UnmanagedByml<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>
}

impl<'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize> UnmanagedByml<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P> {
  fn compute_header_size() -> u64 { BymlHeader::<'static, 0>::size() }

  fn header(&mut self) -> Result<BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE>, GetHeaderError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    self.reader.seek(SeekFrom::Start(0)).unwrap();

    let header: BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE> = self.reader.read(DiagnosticNodeName::from("Header"))
      .map_domain_specific(|v| { GetHeaderError::HeaderError(v) })?
      .map_out_of_bounds(|oob| { GetHeaderError::MissingData(oob) })?
      .map_err(|provider_read| { GetHeaderError::ProviderError(provider_read) })?;

    Ok(header)
  }

  pub fn construct_in() {}

  pub fn version(&mut self) -> Result<u16, GetHeaderError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.header()?.version)
  }

  pub fn set_version(&mut self) -> () {
    todo!()
  }

  pub fn endianness(&mut self) -> Result<Endianness, GetHeaderError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.header()?.endianness)
  }

  pub fn set_endianness(&mut self) -> () {
    todo!()
  }

  pub fn over(provider: P, pool: &'pool DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<UnmanagedByml<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>, LoadError<'pool, P::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let mut unmanaged_byml = UnmanagedByml { reader: Reader::root(provider, Endianness::Big, pool, DiagnosticNodeName::from("BYML")) };
    let header = unmanaged_byml.header()
      .map_err(|e| LoadError::HeaderGetError(e))?;

    unmanaged_byml.reader.set_endianness(header.endianness);

    LoadError::assert_supported(header.version, header.endianness, || header.version_diagnostic())?;

    Ok(unmanaged_byml)
  }
}