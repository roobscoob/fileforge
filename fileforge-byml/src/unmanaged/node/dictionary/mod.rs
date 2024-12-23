use core::{ffi::CStr, panic};

use intx::U24;

use fileforge_lib::{
  diagnostic::node::name::DiagnosticNodeName,
  provider::{error::never::Never, r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::{
    endianness::Endianness,
    error::{
      parse::ParseError, parse_primitive::ParsePrimitiveError,
      underlying_provider_error::UnderlyingProviderError,
      underlying_provider_read::UnderlyingProviderReadError,
      underlying_provider_stat::UnderlyingProviderStatError,
    },
    r#trait::readable::FixedSizeReadable,
    Reader, SeekFrom,
  },
};

pub mod error;
pub mod iterator;

use crate::{
  unmanaged::{
    error::get_string_table::GetStringTableError, string_table::error::get::GetError, BymlReader,
  },
  util::binary_search::{fallible_binary_search, fallible_mapping_binary_search},
};

use self::{
  error::{
    dictionary_get::DictionaryGetError,
    dictionary_get_node_name::DictionaryGetNodeNameError,
    get_length::{DictionaryNotLargeEnough, GetLengthError},
  },
  iterator::BymlDictionaryNodeReaderIterator,
};

use super::{r#trait::BymlNodeReader, BymlReaderNode};

pub struct BymlDictionaryNodeReader<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  includes_remapping_table: bool,
  reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP::DynReturnedProviderType<'byml_provider>>,
  byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

// impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> Clone
//   for BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
// {
//   fn clone(&self) -> Self {
//     BymlDictionaryNodeReader {
//       includes_remapping_table: self.includes_remapping_table,
//       reader: self.reader.clone(),
//       byml: self.byml,
//     }
//   }
// }

#[derive(Clone)]
pub struct BymlDictionaryNodeElement<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  r#type: u8,
  node_name_index: U24,
  value: u32,
  endianness: Endianness,
  byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlDictionaryNodeElement<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  pub fn with_node_name<T>(
    &self,
    mapper: impl for<'s> FnOnce(&'s CStr) -> T,
  ) -> Result<T, DictionaryGetNodeNameError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>> {
    let st = self
      .byml
      .key_table()
      .map_err(|e| DictionaryGetNodeNameError::GetStringTableError(e))?;

    if let Some(mut st) = st {
      st.try_get(self.node_name_index.into(), mapper)
        .map_err(|e| DictionaryGetNodeNameError::GetStringTableEntryError(e))
    } else {
      Err(DictionaryGetNodeNameError::NoStringTable)
    }
  }

  pub fn value(
    &self,
  ) -> BymlReaderNode<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP> {
    BymlReaderNode {
      r#type: self.r#type,
      value: self.value,
      disable_inline: false,
      endianness: self.endianness,
      byml: self.byml,
    }
  }
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, 8>
  for BymlDictionaryNodeElement<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  type Argument = &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>;
  type Error = Never;

  fn read<RP: Provider>(
    reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
    argument: Self::Argument,
  ) -> Result<
    Self,
    fileforge_lib::reader::error::parse::ParseError<
      'pool,
      Self::Error,
      RP::ReadError,
      RP::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let node_name_index: U24 = reader.get("NameIndex")?;
    let r#type: u8 = reader.get("Type")?;
    let node_value: u32 = reader.get("Value")?;

    Ok(BymlDictionaryNodeElement {
      node_name_index,
      value: node_value,
      r#type,
      endianness: reader.endianness(),
      byml: argument,
    })
  }
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  pub(super) fn get_index(
    &mut self,
    index: usize,
  ) -> Result<
    Option<BymlDictionaryNodeElement<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>>,
    UnderlyingProviderError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    match self.reader.seek(SeekFrom::Start(4 + (index as u64 * 8))) {
      Ok(_) => {}
      Err(_) => return Ok(None),
    }

    match self
      .reader
      .read_with(DiagnosticNodeName::from_index(index as u64), self.byml)
    {
      Ok(v) => Ok(Some(v)),
      Err(ParseError::OutOfBounds(_)) => Ok(None),
      Err(ParseError::DomainSpecific(Never)) => unreachable!(),
      Err(ParseError::UnderlyingProviderReadError(upre)) => {
        Err(UnderlyingProviderError::ReadError(upre))
      }
      Err(ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(se))) => Err(
        UnderlyingProviderError::StatError(UnderlyingProviderStatError(se)),
      ),
    }
  }

  pub fn into_iter(
    self,
  ) -> BymlDictionaryNodeReaderIterator<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
  {
    BymlDictionaryNodeReaderIterator {
      index: 0,
      reader: self,
    }
  }

  pub fn get(
    &mut self,
    key: &CStr,
  ) -> Result<
    Option<BymlReaderNode<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>>,
    DictionaryGetError<
      'byml_provider,
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      <DynamicSliceProvider<'_, BP::DynReturnedProviderType<'byml_provider>> as Provider>::ReadError,
      <DynamicSliceProvider<'_, BP::DynReturnedProviderType<'byml_provider>> as Provider>::StatError,
      BP,
    >,
  >{
    let key_index = match self
      .byml
      .key_table()
      .map_err(|e| DictionaryGetError::GetStringTableError(e))?
    {
      None => return Ok(None),
      Some(v) => v,
    }
    .try_get_index(key)
    .map_err(|e| DictionaryGetError::StringTableGetIndexError(e))?;

    if key_index.is_none() {
      return Ok(None);
    }

    let key_index = key_index.unwrap();

    let length = self
      .length()
      .map_err(|e| DictionaryGetError::GetLengthError(e))?;

    let index = fallible_mapping_binary_search(
      length as usize,
      |index| self.get_index(index).map(|v| v.unwrap()),
      |e| key_index.cmp(&e.node_name_index.into()),
    )
    .map_err(|e| DictionaryGetError::UnderlyingProviderError(e))?;

    if index.is_none() {
      return Ok(None);
    }

    let element = index.unwrap();

    Ok(Some(BymlReaderNode {
      byml: self.byml,
      disable_inline: false,
      endianness: self.reader.endianness(),
      r#type: element.r#type,
      value: element.value,
    }))
  }

  pub fn length(
    &mut self,
  ) -> Result<
    u32,
    GetLengthError<
      'pool,
      DynamicSliceProvider<'byml_provider, BP::DynReturnedProviderType<'byml_provider>>,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let length: U24 = self
      .reader
      .get_at("Length", 1)
      .map_err(|_| match self.reader.len() {
        Ok(available_length) => GetLengthError::NotLargeEnough(DictionaryNotLargeEnough {
          desired_length: 4,
          available_length,
        }),
        Err(e) => GetLengthError::UnderlyingProviderError(UnderlyingProviderError::StatError(
          UnderlyingProviderStatError(e),
        )),
      })?;

    Ok(length.into())
  }
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
  for BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  type ReadError = ();

  fn type_id_supported(type_id: u8) -> bool { type_id == 0xC1 || type_id == 0xC4 }

  fn requires_dereferencing(_: u8) -> bool { true }

  fn from_reader(
    type_id: u8,
    reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP::DynReturnedProviderType<'byml_provider>>,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Result<Self, Self::ReadError> {
    Ok(Self {
      includes_remapping_table: type_id == 0xC4,
      reader,
      byml,
    })
  }

  fn from_value(
    _type_id: u8,
    _value: u32,
    _byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self {
    panic!("Cannot create from value");
  }
}
