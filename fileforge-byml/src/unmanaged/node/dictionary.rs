use core::ffi::CStr;

use intx::U24;

use fileforge_lib::{
  provider::{r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::{Reader, SeekFrom},
};

use crate::{unmanaged::BymlReader, util::binary_search::fallible_binary_search};

use super::{
  error::{
    dictionary_get::DictionaryGetError,
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
  RP: Provider,
  BP: Provider,
> {
  includes_remapping_table: bool,
  reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
  byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
}

impl<
    'byml,
    'byml_provider,
    'pool,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
    RP: Provider,
    BP: Provider,
  > BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>
{
  pub fn get(
    &mut self,
    key: &CStr,
  ) -> Result<
    Option<BymlReaderNode<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>>,
    DictionaryGetError<
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      RP,
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

    let index = fallible_binary_search(length as usize, |index| {
      self
        .reader
        .seek(SeekFrom::Start(5 + (index as u64 * 8)))
        .map_err(|_| DictionaryGetError::EntryOutOfBounds())?;

      let node_name_index: U24 = self
        .reader
        .get("NameIndex")
        .map_err(|_| DictionaryGetError::EntryOutOfBounds())?;

      Ok(key_index.cmp(&node_name_index.into()))
    })?;

    if index.is_none() {
      return Ok(None);
    }

    let index = index.unwrap();

    todo!()
  }

  pub fn length(&mut self) -> Result<u32, GetLengthError<'pool, RP, DIAGNOSTIC_NODE_NAME_SIZE>> {
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

impl<
    'byml,
    'byml_provider,
    'pool,
    const DIAGNOSTIC_NODE_NAME_SIZE: usize,
    RP: Provider,
    BP: Provider,
  > BymlNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>
  for BymlDictionaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP, BP>
{
  fn type_id_supported(type_id: u8) -> bool { type_id == 0xC1 || type_id == 0xC4 }

  fn requires_dereferencing(_: u8) -> bool { true }

  fn from_reader(
    type_id: u8,
    reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self {
    Self {
      includes_remapping_table: type_id == 0xC4,
      reader,
      byml,
    }
  }
}
