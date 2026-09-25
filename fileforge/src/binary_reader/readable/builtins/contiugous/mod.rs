use fileforge_macros::{story, text};
use core::{convert::Infallible, marker::PhantomData};

use crate::{
  binary_reader::{
    self,
    error::common::LOW_LEVEL_ERROR,
    readable::{builtins::array::ArrayReadError, IntoReadable, Readable, RefReadable},
    BinaryReader,
  },
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
    },
    report::Report,
    FileforgeError,
  },
  stream::{self, ReadableStream},
};

pub struct Contiguous<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> {
  reader: BinaryReader<'pool, S>,
  index: u64,
  generator: Gen,
  _phantom: PhantomData<fn() -> T>,
}

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> Contiguous<'pool, S, T, Gen> {
  pub async fn finish(mut self, length: u64) -> Result<(), stream::StreamSkipError<ContiguousSkipError<'pool, <S as ReadableStream>::SkipError, <T as Readable<'pool, S>>::Error>>> {
    self.skip(length.saturating_sub(self.index)).await
  }
}

impl<'s, 'pool: 's, S: ReadableStream<Type = u8>, T: Readable<'pool, &'s mut S> + 's, Gen: 's + FnMut(u64) -> T::Argument> RefReadable<'s, 'pool, S> for Contiguous<'pool, &'s mut S, T, Gen> {
  type Error = Infallible;

  type Argument = Gen;

  async fn read_ref(reader: &'s mut BinaryReader<'pool, S>, generator: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Self {
      reader: reader.borrow_fork(),
      index: 0,
      generator,
      _phantom: PhantomData,
    })
  }
}

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> IntoReadable<'pool, S> for Contiguous<'pool, S, T, Gen> {
  type Error = Infallible;

  type Argument = Gen;

  async fn read(reader: BinaryReader<'pool, S>, generator: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Self {
      reader,
      index: 0,
      generator,
      _phantom: PhantomData,
    })
  }
}

#[story("element failed to read", {
  let error: ContiguousSkipError<'_, StoryUserError, _> = ContiguousSkipError::Read {
    index: 2,
    read_error: read_exhausted::<u32, StoryUserError>(dr!("save.bin" @ 0..10), 8, DiagnosticValue(10, None)),
  };
  error
})]
#[story("skip distance overflowed", ContiguousSkipError::<StoryUserError, StoryUserError>::Overflowed)]
#[story("stream failed", ContiguousSkipError::<StoryUserError, StoryUserError>::Stream(binary_reader::SkipError::User(StoryUserError)))]
pub enum ContiguousSkipError<'pool, S: stream::UserSkipError, R: FileforgeError> {
  Overflowed, // todo: item size + size
  Read { index: u64, read_error: R },
  Stream(binary_reader::SkipError<'pool, S>),
}
const SKIP_OVERFLOWED: ConstText = ConstText::new("Skipping that many items would go past the largest possible position.", &REPORT_ERROR_TEXT);
const SKIPPING_ITEMS: ConstText = ConstText::new("This happened while skipping over items.", &REPORT_INFO_LINE_TEXT);

impl<'pool, S: stream::UserSkipError, R: FileforgeError> FileforgeError for ContiguousSkipError<'pool, S, R> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Overflowed => Report::new::<Self>(provider, &"Skip overflowed")
        .with_info_line(&SKIP_OVERFLOWED)
        .with_flag_line(LOW_LEVEL_ERROR)
        .apply(callback),

      Self::Read { index, read_error } => read_error.render_into_report(provider, |report| {
        let index = FormattedUnsigned::new(*index as u128).separator(3, ",");
        let context = text!([&REPORT_INFO_LINE_TEXT] "This happened while skipping over the item at index {&index}.");

        report.with_info_line(&context).apply(callback)
      }),

      Self::Stream(error) => error.render_into_report(provider, |report| report.with_info_line(&SKIPPING_ITEMS).apply(callback)),
    }
  }
}
impl<'pool, S: stream::UserSkipError, E: FileforgeError> stream::UserSkipError for ContiguousSkipError<'pool, S, E> {}

impl<'pool, S: ReadableStream<Type = u8>, T: Readable<'pool, S>, Gen: FnMut(u64) -> T::Argument> ReadableStream for Contiguous<'pool, S, T, Gen> {
  type Type = T;

  type ReadError = ArrayReadError<T::Error>;

  type SkipError = ContiguousSkipError<'pool, S::SkipError, T::Error>;

  fn offset(&self) -> u64 {
    self.index as u64
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, stream::StreamReadError<Self::ReadError>> {
    let arguments = core::array::from_fn(|index| (self.generator)(self.index + index as u64));
    let actual = self.reader.read_with::<[T; SIZE]>(arguments).await.map_err(stream::StreamReadError::User)?;

    self.index += SIZE as u64;

    Ok(reader(&actual).await)
  }

  async fn skip(&mut self, size: u64) -> Result<(), stream::StreamSkipError<Self::SkipError>> {
    // ensure that we can skip `size` items
    let end_index = self.index.checked_add(size).ok_or(stream::StreamSkipError::User(ContiguousSkipError::Overflowed))?;

    if let Some(item_size) = T::SIZE {
      let total_size = size.checked_mul(item_size).ok_or(stream::StreamSkipError::User(ContiguousSkipError::Overflowed))?;
      self.reader.skip(total_size).await.map_err(ContiguousSkipError::Stream).map_err(stream::StreamSkipError::User)?;

      // only once the skip succeeded, so a failed skip leaves the index where the reader is
      self.index = end_index;
      Ok(())
    } else {
      while self.index < end_index {
        self
          .reader
          .read_with::<T>((self.generator)(self.index))
          .await
          .map_err(|error| ContiguousSkipError::Read {
            index: self.index,
            read_error: error,
          })
          .map_err(stream::StreamSkipError::User)?;

        // one at a time, so a failed read leaves the index at the item that failed
        self.index += 1;
      }

      Ok(())
    }
  }
}

#[cfg(test)]
mod tests {
  use std::vec;

  use super::Contiguous;
  use crate::{
    binary_reader::{endianness::Endianness, BinaryReader},
    provider::hint::ReadHint,
    stream::{ReadableStream, SINGLE},
  };

  #[tokio::test]
  async fn a_failed_fixed_size_skip_leaves_the_index_unchanged() {
    // Three u32s.
    let reader = BinaryReader::new_from_provider(vec![0u8, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3], Endianness::BigEndian, ReadHint::new());
    let mut items = reader.into_with::<Contiguous<_, u32, _>>(|_| ()).await.unwrap();

    assert!(items.skip(1).await.is_ok());
    assert_eq!(items.offset(), 1);

    assert!(items.skip(5).await.is_err());
    assert_eq!(items.offset(), 1);
    assert_eq!(items.read(SINGLE).await.ok(), Some(2), "the reader should still be at item 1");
  }

  #[tokio::test]
  async fn a_failed_variable_size_skip_stops_at_the_item_that_failed() {
    // Two whole [u8; 2] items, then half of a third.
    let reader = BinaryReader::new_from_provider(vec![1u8, 2, 3, 4, 5], Endianness::BigEndian, ReadHint::new());
    let mut items = reader.into_with::<Contiguous<_, [u8; 2], _>>(|_| [(); 2]).await.unwrap();

    assert!(items.skip(3).await.is_err());
    assert_eq!(items.offset(), 2);
  }
}
