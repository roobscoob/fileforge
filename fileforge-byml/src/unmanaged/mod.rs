use error::{
  get_root_node::GetRootNodeError,
  get_string_table::{GetStringTableError, StringTableOutOfBounds},
  header_root_node_data_out_of_bounds::HeaderRootNodeOutOfBounds,
};
use node::BymlReaderNode;
use string_table::StringTable;

use fileforge_lib::{
  diagnostic::{
    node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference},
    pool::DiagnosticPool,
  },
  provider::{r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::{
    endianness::Endianness,
    error::{
      parse::ParseErrorResultExtension, parse_primitive::ParsePrimitiveErrorResultExtension,
      underlying_provider_stat::UnderlyingProviderStatError,
    },
    Reader, SeekFrom,
  },
};

use self::{
  error::{get_header::GetHeaderError, load::LoadError},
  header::BymlHeader,
};

pub mod error;
pub mod header;
pub mod node;
pub mod string_table;

pub struct BymlReader<'provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  provider: &'provider P,
  pool: &'pool &'pool mut DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
}

impl<'provider, 'pool, P: Provider, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  BymlReader<'provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>
{
  fn compute_header_size() -> u64 { BymlHeader::<'static, 0>::size() }

  fn diagnostic_root(
    &self,
  ) -> Result<
    DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    UnderlyingProviderStatError<P::StatError>,
  > {
    Ok(self.pool.try_create(
      DiagnosticBranch::None,
      None,
      DiagnosticNodeName::from("BYML"),
    ))
  }

  fn header(
    &self,
  ) -> Result<
    BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    GetHeaderError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    let reader = Reader::<'pool, DIAGNOSTIC_NODE_NAME_SIZE, &P>::at(
      &*self.provider,
      Endianness::Big,
      self
        .diagnostic_root()
        .unwrap_or_else(|_| DiagnosticReference::new_invalid_from_pool(self.pool)),
    );

    let header: BymlHeader<'pool, DIAGNOSTIC_NODE_NAME_SIZE> = reader
      .read(DiagnosticNodeName::from("Header"))
      .map_domain_specific(|v| GetHeaderError::HeaderError(v))?
      .map_out_of_bounds(|oob| GetHeaderError::MissingData(oob))?
      .map_err(|provider_read| GetHeaderError::ProviderError(provider_read))?;

    Ok(header)
  }

  pub fn construct_in() {}

  pub fn version(
    &self,
  ) -> Result<u16, GetHeaderError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.header()?.version)
  }

  pub fn endianness(
    &self,
  ) -> Result<
    Endianness,
    GetHeaderError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    Ok(self.header()?.endianness)
  }

  pub fn over(
    provider: &'provider P,
    pool: &'pool &'pool mut DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> BymlReader<'provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, P> {
    BymlReader { provider, pool }
  }

  pub fn string_table(
    &self,
  ) -> Result<
    Option<
      StringTable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, <P as Provider>::DynReturnedProviderType<'_>>,
    >,
    GetStringTableError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    let header = self
      .header()
      .map_err(|e| GetStringTableError::GetHeaderError(e))?;

    if header.string_table_offset == 0 {
      return Ok(None);
    }

    let mut reader = Reader::<'pool, DIAGNOSTIC_NODE_NAME_SIZE, &P>::root(
      &*self.provider,
      Endianness::Big,
      *self.pool,
      DiagnosticNodeName::from("Byml"),
    );

    reader
      .seek(SeekFrom::Start(header.string_table_offset as u64))
      .map_err(|_| {
        GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
          string_table_parent: reader.diagnostic_reference(),
          string_table_size: None,
          string_table_size_complete: false,
          string_table_position: header.string_table_offset as usize,
          string_table_position_dr: header.string_table_offset_diagnostic(),
          string_table_size_dr: None,
          byml_size: self
            .provider
            .len()
            .map_err(|e| UnderlyingProviderStatError(e)),
        })
      })?;

    let (size, drb) = StringTable::size(
      &mut reader,
      || {
        self
          .provider
          .len()
          .map_err(|e| UnderlyingProviderStatError(e))
      },
      || header.string_table_offset_diagnostic(),
    )
    .map_err(|e| GetStringTableError::GetStringTableSizeError(e))?;

    let dr = reader.diagnostic_reference().create_physical_child(
      header.string_table_offset as u64,
      Some(size),
      DiagnosticNodeName::from("String Table"),
    );

    let provider = self
      .provider
      .slice_dyn(header.string_table_offset as u64, Some(size))
      .map_err(|_| {
        GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
          string_table_parent: reader.diagnostic_reference(),
          string_table_position: header.string_table_offset as usize,
          string_table_position_dr: header.string_table_offset_diagnostic(),
          string_table_size: Some(size as usize),
          string_table_size_complete: true,
          string_table_size_dr: Some(drb),
          byml_size: self
            .provider
            .len()
            .map_err(|e| UnderlyingProviderStatError(e)),
        })
      })?;

    let reader = Reader::at(provider, header.endianness, dr);

    Ok(Some(StringTable { reader }))
  }

  pub fn key_table(
    &self,
  ) -> Result<
    Option<
      StringTable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, <P as Provider>::DynReturnedProviderType<'_>>,
    >,
    GetStringTableError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    let header = self
      .header()
      .map_err(|e| GetStringTableError::GetHeaderError(e))?;

    if header.string_table_offset == 0 {
      return Ok(None);
    }

    let mut reader = Reader::<'pool, DIAGNOSTIC_NODE_NAME_SIZE, &P>::root(
      &*self.provider,
      Endianness::Big,
      *self.pool,
      DiagnosticNodeName::from("Byml"),
    );

    reader
      .seek(SeekFrom::Start(header.key_table_offset as u64))
      .map_err(|_| {
        GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
          string_table_parent: reader.diagnostic_reference(),
          string_table_size: None,
          string_table_size_complete: false,
          string_table_position: header.key_table_offset as usize,
          string_table_position_dr: header.key_table_offset_diagnostic(),
          string_table_size_dr: None,
          byml_size: self
            .provider
            .len()
            .map_err(|e| UnderlyingProviderStatError(e)),
        })
      })?;

    let (size, drb) = StringTable::size(
      &mut reader,
      || {
        self
          .provider
          .len()
          .map_err(|e| UnderlyingProviderStatError(e))
      },
      || header.key_table_offset_diagnostic(),
    )
    .map_err(|e| GetStringTableError::GetStringTableSizeError(e))?;

    let dr = reader.diagnostic_reference().create_physical_child(
      header.key_table_offset as u64,
      Some(size),
      DiagnosticNodeName::from("String Table"),
    );

    let provider = self
      .provider
      .slice_dyn(header.key_table_offset as u64, Some(size))
      .map_err(|_| {
        GetStringTableError::StringTableOutOfBounds(StringTableOutOfBounds {
          string_table_parent: reader.diagnostic_reference(),
          string_table_position: header.key_table_offset as usize,
          string_table_position_dr: header.key_table_offset_diagnostic(),
          string_table_size: Some(size as usize),
          string_table_size_complete: true,
          string_table_size_dr: Some(drb),
          byml_size: self
            .provider
            .len()
            .map_err(|e| UnderlyingProviderStatError(e)),
        })
      })?;

    let reader = Reader::at(provider, header.endianness, dr);

    Ok(Some(StringTable { reader }))
  }

  pub fn root<'s>(
    &'s self,
  ) -> Result<
    Option<BymlReaderNode<'s, 'provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>>,
    GetRootNodeError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    let header = self
      .header()
      .map_err(|e| GetRootNodeError::GetHeaderError(e))?;

    if header.root_data_offset == 0 {
      return Ok(None);
    }

    let r#type = self
      .provider
      .with_read(header.root_data_offset.into(), |v: &[u8; 1]| v[0])
      .map_err(|_| GetRootNodeError::ReadErrorWhileReadingType())?
      .map_err(|_| GetRootNodeError::HederRootNodeOutOfBounds(HeaderRootNodeOutOfBounds {}))?;

    Ok(Some(BymlReaderNode {
      byml: self,
      disable_inline: true,
      endianness: header.endianness,
      r#type,
      value: header.root_data_offset,
    }))
  }
}
