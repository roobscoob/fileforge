use core::ffi::CStr;

use intx::U24;

use fileforge_lib::{
  provider::{r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::{
    error::{
      parse_primitive::ParsePrimitiveError, underlying_provider_read::UnderlyingProviderReadError,
    },
    Reader, SeekFrom,
  },
};

use crate::{
  unmanaged::{error::get_string_table::GetStringTableError, BymlReader},
  util::binary_search::{fallible_binary_search, fallible_mapping_binary_search},
};

use super::{
  dictionary_iterator::BymlDictionaryNodeReaderIterator,
  error::{
    dictionary_get::DictionaryGetError,
    dictionary_get_node_name::DictionaryGetNodeNameError,
    get_length::{DictionaryNotLargeEnough, GetLengthError},
  },
  r#trait::BymlNodeReader,
  BymlReaderNode,
};

pub struct BymlDictionaryNodeReader<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  includes_remapping_table: bool,
  reader: Reader<
    'pool,
    DIAGNOSTIC_NODE_NAME_SIZE,
    DynamicSliceProvider<'byml_provider, BP::DynReturnedProviderType>,
  >,
  byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> Clone
  for BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  fn clone(&self) -> Self {
    BymlDictionaryNodeReader {
      includes_remapping_table: self.includes_remapping_table,
      reader: self.reader.clone(),
      byml: self.byml,
    }
  }
}

#[derive(Clone)]
pub struct BymlDictionaryNodeElement<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
> {
  pub r#type: u8,
  pub node_name_index: U24,
  pub value: u32,
  byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlDictionaryNodeElement<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  pub fn with_node_name<T>(
    &self,
    mapper: impl for<'s> FnOnce(&'s CStr) -> T,
  ) -> Result<T, DictionaryGetNodeNameError<'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>> {
    let mut st = self
      .byml
      .key_table()
      .map_err(|e| DictionaryGetNodeNameError::GetStringTableError(e))?;

    st.try_get(self.node_name_index.into(), mapper)
      .map_err(|e| DictionaryGetNodeNameError::GetStringTableEntryError(e))
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
    UnderlyingProviderReadError<'pool, BP::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    match self.reader.seek(SeekFrom::Start(4 + (index as u64 * 8))) {
      Ok(_) => {}
      Err(_) => return Ok(None),
    }

    let node_name_index: U24 = match self.reader.get("NameIndex") {
      Ok(v) => v,
      Err(ParsePrimitiveError::OutOfBounds(_)) => return Ok(None),
      Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)) => return Err(upre),
    };

    let r#type: u8 = match self.reader.get("Type") {
      Ok(v) => v,
      Err(ParsePrimitiveError::OutOfBounds(_)) => return Ok(None),
      Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)) => return Err(upre),
    };

    let node_value: u32 = match self.reader.get("Value") {
      Ok(v) => v,
      Err(ParsePrimitiveError::OutOfBounds(_)) => return Ok(None),
      Err(ParsePrimitiveError::UnderlyingProviderReadError(upre)) => return Err(upre),
    };

    Ok(Some(BymlDictionaryNodeElement {
      node_name_index,
      value: node_value,
      r#type,
      byml: self.byml,
    }))
  }

  pub fn iter(
    &self,
  ) -> BymlDictionaryNodeReaderIterator<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
  {
    BymlDictionaryNodeReaderIterator {
      index: 0,
      reader: self.clone(),
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
      <DynamicSliceProvider<'_, BP::DynReturnedProviderType> as Provider>::ReadError,
      BP,
    >,
  > {
    let key_index = self
      .byml
      .key_table()
      .map_err(|e| DictionaryGetError::GetStringTableError(e))?
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
    .map_err(|e| DictionaryGetError::UnderlyingProviderReadError(e))?;

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
      DynamicSliceProvider<'byml_provider, BP::DynReturnedProviderType>,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    self.reader.seek(SeekFrom::Start(1)).map_err(|_| {
      GetLengthError::NotLargeEnough(DictionaryNotLargeEnough {
        desired_length: 1,
        available_length: self.reader.len(),
      })
    })?;

    let length: U24 = self.reader.get("Length").map_err(|_| {
      GetLengthError::NotLargeEnough(DictionaryNotLargeEnough {
        desired_length: 4,
        available_length: self.reader.len(),
      })
    })?;

    Ok(length.into())
  }
}

impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
  BymlNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
  for BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
{
  fn type_id_supported(type_id: u8) -> bool { type_id == 0xC1 || type_id == 0xC4 }

  fn requires_dereferencing(_: u8) -> bool { true }

  fn from_reader(
    type_id: u8,
    reader: Reader<
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      DynamicSliceProvider<'byml_provider, BP::DynReturnedProviderType>,
    >,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self {
    Self {
      includes_remapping_table: type_id == 0xC4,
      reader,
      byml,
    }
  }

  fn from_value(
    _type_id: u8,
    _value: u32,
    _byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self {
    panic!("Cannot create from value");
  }
}
