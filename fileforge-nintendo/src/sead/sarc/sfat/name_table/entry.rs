use core::hash::Hasher;

use fileforge::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::ReadableStream,
};

use crate::sead::sarc::sfat::name_table::hasher::SfntHasher;

pub struct NameTableEntry<'r, 'pool, UnderlyingStream: ReadableStream<Type = u8>> {
  pub(super) hasher: SfntHasher,
  pub(super) stream: &'r mut BinaryReader<'pool, UnderlyingStream>,
}

impl<'r, 'pool, UnderlyingStream: ReadableStream<Type = u8>> NameTableEntry<'r, 'pool, UnderlyingStream> {
  pub async fn into_hash(self) -> Result<u32, Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, <UnderlyingStream as ReadableStream>::ReadError>>> {
    let mut hasher = self.hasher.clone();

    loop {
      let chunk: [u8; 4] = self.stream.get().await?;
      let non_null = chunk.split(|&v| v == 0).next().unwrap();

      hasher.write(&non_null);

      if non_null.len() != 4 {
        break;
      }
    }

    Ok(hasher.get_hash())
  }
}
