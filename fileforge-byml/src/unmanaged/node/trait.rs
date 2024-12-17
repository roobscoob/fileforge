use fileforge_lib::{
  provider::{r#trait::Provider, slice::dynamic::DynamicSliceProvider},
  reader::Reader,
};

use crate::unmanaged::BymlReader;

pub trait BymlNodeReader<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  BP: Provider,
>: Sized
{
  fn type_id_supported(type_id: u8) -> bool;
  fn requires_dereferencing(type_id: u8) -> bool;

  fn from_reader(
    type_id: u8,
    reader: Reader<
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      DynamicSliceProvider<'byml_provider, <BP as Provider>::DynReturnedProviderType>,
    >,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self;

  fn from_value(
    type_id: u8,
    value: u32,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self;
}
