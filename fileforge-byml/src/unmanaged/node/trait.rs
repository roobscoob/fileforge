use fileforge_lib::{provider::r#trait::Provider, reader::Reader};

use crate::unmanaged::BymlReader;

pub trait BymlNodeReader<
  'byml,
  'byml_provider,
  'pool,
  const DIAGNOSTIC_NODE_NAME_SIZE: usize,
  RP: Provider,
  BP: Provider,
>: Sized
{
  fn type_id_supported(type_id: u8) -> bool;
  fn requires_dereferencing(type_id: u8) -> bool;

  fn from_reader(
    type_id: u8,
    reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, RP>,
    byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
  ) -> Self;
}
