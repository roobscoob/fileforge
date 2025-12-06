use fileforge::{
  binary_reader::{
    error::{primitive_name_annotation::PrimitiveName, RewindError, SetPrimitiveError, SkipError},
    mutable::Mutable,
    BinaryReader, PrimitiveWriter,
  },
  error::ext::annotations::annotated::Annotated,
  stream::{MutableStream, RewindableStream},
};
use fileforge_std::magic::{Magic, MagicError};

use super::{readable::YAZ0_MAGIC, Yaz0Header};

// FIELD INDEXES:
// 0: Right after the magic
// 1: Right after the uncompressed size
// 2: Right after the alignment
// 3: Right after the unused. No more fields are writable.
pub struct Yaz0HeaderMutator<'pool, 'l, S: MutableStream<Type = u8> + 'l, const FIELD_INDEX: usize> {
  reader: &'l mut BinaryReader<'pool, S>,
}

impl<'pool, 'l, S: MutableStream<Type = u8> + 'l> Yaz0HeaderMutator<'pool, 'l, S, 0> {
  pub async fn with_uncompressed_size(
    self,
    size: u32,
  ) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 1>, Annotated<PrimitiveName<fileforge::binary_reader::error::common::Write>, SetPrimitiveError<'pool, <S as MutableStream>::MutateError>>> {
    self.reader.set(size).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }

  pub async fn keep_uncompressed_size(self) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 1>, SkipError<'pool, S::SkipError>> {
    self.reader.skip(4).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }
}

impl<'pool, 'l, S: MutableStream<Type = u8> + 'l> Yaz0HeaderMutator<'pool, 'l, S, 1> {
  pub async fn back(self) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 0>, RewindError<'pool, S::RewindError>>
  where
    S: RewindableStream,
  {
    self.reader.rewind(4).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }

  pub async fn with_alignment(
    self,
    alignment: u32,
  ) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 2>, Annotated<PrimitiveName<fileforge::binary_reader::error::common::Write>, SetPrimitiveError<'pool, <S as MutableStream>::MutateError>>> {
    self.reader.set(alignment).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }

  pub async fn keep_alignment(self) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 2>, SkipError<'pool, S::SkipError>> {
    self.reader.skip(4).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }
}

impl<'pool, 'l, S: MutableStream<Type = u8> + 'l> Yaz0HeaderMutator<'pool, 'l, S, 2> {
  pub async fn back(self) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 1>, RewindError<'pool, S::RewindError>>
  where
    S: RewindableStream,
  {
    self.reader.rewind(4).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }

  pub async fn start(self) -> Result<Yaz0HeaderMutator<'pool, 'l, S, 0>, RewindError<'pool, S::RewindError>>
  where
    S: RewindableStream,
  {
    self.reader.rewind(8).await?;
    Ok(Yaz0HeaderMutator { reader: self.reader })
  }
}

impl<'pool, S: MutableStream<Type = u8>> Mutable<'pool, S> for Yaz0Header {
  type Mutator<'l>
    = Yaz0HeaderMutator<'pool, 'l, S, 0>
  where
    'pool: 'l,
    Self: 'l,
    S: 'l;

  type Error = MagicError<'pool, 4, S::ReadError>;

  async fn mutate<'l>(reader: &'l mut BinaryReader<'pool, S>) -> Result<Self::Mutator<'l>, Self::Error>
  where
    Self: 'l,
  {
    reader.read_with::<Magic<4>>(YAZ0_MAGIC).await?;

    Ok(Yaz0HeaderMutator { reader })
  }
}
