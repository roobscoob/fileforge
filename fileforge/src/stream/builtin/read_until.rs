use fileforge_macros::story;
use crate::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
    FileforgeError,
  },
  stream::{
    self,
    error::{stream_exhausted::StreamExhaustedError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError},
    ReadableStream, StreamReadError, StreamSkipError, SINGLE,
  },
};

/// The elements of a stream up to (not including) the first `needle`, such as a null-terminated
/// string. Its length isn't known until the needle is found, so it reads the underlying stream one
/// element at a time and never consumes anything past the needle.
pub struct ReadUntil<R: ReadableStream> {
  stream: R,
  needle: R::Type,
  /// How many elements have been consumed, counted from the start of this stream.
  offset: u64,
  /// How many elements come before the needle, once it has been found.
  length: Option<u64>,
}

impl<R: ReadableStream> ReadUntil<R> {
  pub fn new(stream: R, needle: R::Type) -> Self {
    Self {
      stream,
      needle,
      offset: 0,
      length: None,
    }
  }

  /// The underlying stream. Once the needle has been found, it sits just after it.
  pub fn into_inner(self) -> R {
    self.stream
  }
}

#[story("read failed while searching for the needle", ReadUntilSkipError::<StoryUserError>(StreamReadError::User(StoryUserError)))]
#[story("stream ended while searching for the needle", ReadUntilSkipError::<StoryUserError>(StreamReadError::StreamExhausted(
  crate::stream::error::stream_exhausted::StreamExhaustedError { stream_length: 16, read_length: 1, read_offset: 16 },
)))]
#[derive(Debug)]
pub struct ReadUntilSkipError<E: stream::UserReadError>(StreamReadError<E>);

const SKIPPING_TO_TERMINATOR: ConstText = ConstText::new(
  "This happened while skipping ahead in a value that ends at a terminator, such as a null-terminated string.",
  &REPORT_INFO_LINE_TEXT,
);

impl<E: stream::UserReadError> FileforgeError for ReadUntilSkipError<E> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    self.0.render_into_report(provider, |report| report.with_info_line(&SKIPPING_TO_TERMINATOR).apply(callback))
  }
}

impl<E: stream::UserReadError> stream::UserSkipError for ReadUntilSkipError<E> {}

impl<R: ReadableStream> ReadableStream for ReadUntil<R>
where
  R::Type: PartialEq + Copy,
{
  type Type = R::Type;
  type ReadError = R::ReadError;
  type SkipError = ReadUntilSkipError<Self::ReadError>;

  fn len(&self) -> Option<u64> {
    self.length
  }

  fn remaining(&self) -> Option<u64> {
    self.length.map(|length| length - self.offset)
  }

  fn offset(&self) -> u64 {
    self.offset
  }

  /// Reads the underlying stream one element at a time: a larger read could consume elements past
  /// the needle. A read that runs into the needle fails, leaving the stream at its end.
  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let read_offset = self.offset;
    let exhausted = |stream_length| {
      StreamReadError::StreamExhausted(StreamExhaustedError {
        stream_length,
        read_length: SIZE as u64,
        read_offset,
      })
    };

    if let Some(length) = self.length.filter(|&length| read_offset + SIZE as u64 > length) {
      return Err(exhausted(length));
    }

    let mut buffer = heapless::Vec::<R::Type, SIZE>::new();

    while !buffer.is_full() {
      let item = self.stream.read(SINGLE).await?;

      if item == self.needle {
        self.length = Some(self.offset);
        return Err(exhausted(self.offset));
      }

      let _ = buffer.push(item);
      self.offset += 1;
    }

    let Ok(items) = buffer.into_array::<SIZE>() else { unreachable!("the buffer is full") };

    Ok(reader(&items).await)
  }

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<ReadUntilSkipError<Self::ReadError>>> {
    let seek_point = self.offset.saturating_add(size);
    let out_of_bounds = |stream_length| StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError { stream_length, seek_point });

    if let Some(length) = self.length.filter(|&length| seek_point > length) {
      return Err(out_of_bounds(length));
    }

    while self.offset < seek_point {
      let item = self.stream.read(SINGLE).await.map_err(|e| StreamSkipError::User(ReadUntilSkipError(e)))?;

      if item == self.needle {
        self.length = Some(self.offset);
        return Err(out_of_bounds(self.offset));
      }

      self.offset += 1;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    provider::hint::ReadHint,
    stream::{
      builtin::provider::ProviderStream,
      error::{stream_exhausted::StreamExhaustedError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError},
      extensions::readable::ReadableStreamExt,
      ReadableStream, StreamReadError, StreamSkipError, SINGLE,
    },
  };

  fn stream(bytes: &'static [u8]) -> ProviderStream<&'static [u8]> {
    ProviderStream::new(bytes, ReadHint::new())
  }

  #[tokio::test]
  async fn reads_up_to_the_needle() {
    let mut string = stream(b"abc\0def").read_until(0);

    assert_eq!(string.len(), None);
    assert_eq!(string.read(async |bytes: &[u8; 2]| *bytes).await.unwrap(), *b"ab");
    assert_eq!(string.read(SINGLE).await.unwrap(), b'c');

    let error = string.read(SINGLE).await.unwrap_err();
    assert!(matches!(error, StreamReadError::StreamExhausted(StreamExhaustedError { stream_length: 3, read_length: 1, read_offset: 3 })));
    assert_eq!((string.offset(), string.len(), string.remaining()), (3, Some(3), Some(0)));
  }

  #[tokio::test]
  async fn a_read_across_the_needle_fails_without_reading_past_it() {
    let mut string = stream(b"abc\0def").read_until(0);

    let error = string.read(async |bytes: &[u8; 4]| *bytes).await.unwrap_err();
    assert!(matches!(error, StreamReadError::StreamExhausted(StreamExhaustedError { stream_length: 3, read_length: 4, read_offset: 0 })));
    assert_eq!((string.offset(), string.len()), (3, Some(3)));

    let mut rest = string.into_inner();
    assert_eq!(rest.read(SINGLE).await.unwrap(), b'd');
  }

  #[tokio::test]
  async fn the_offset_counts_from_the_start_of_the_string() {
    let mut underlying = stream(b"xyabc\0");
    underlying.skip(2).await.unwrap();

    let mut string = underlying.read_until(0);
    assert_eq!(string.offset(), 0);

    string.read(SINGLE).await.unwrap();
    assert_eq!(string.offset(), 1);
  }

  #[tokio::test]
  async fn skipping() {
    let mut string = stream(b"abc\0def").read_until(0);

    string.skip(2).await.unwrap();
    assert_eq!(string.read(SINGLE).await.unwrap(), b'c');

    let mut string = stream(b"abc\0def").read_until(0);
    let error = string.skip(5).await.unwrap_err();
    assert!(matches!(error, StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError { stream_length: 3, seek_point: 5 })));
    assert_eq!((string.offset(), string.len()), (3, Some(3)));

    // Now that the length is known, a skip past it fails without reading.
    assert!(string.skip(1).await.is_err());
    assert_eq!(string.offset(), 3);
  }

  #[tokio::test]
  async fn an_empty_string() {
    let mut string = stream(b"\0abc").read_until(0);

    assert!(string.read(SINGLE).await.is_err());
    assert_eq!(string.len(), Some(0));
  }

  #[tokio::test]
  async fn a_missing_needle_is_an_error() {
    let mut string = stream(b"abc").read_until(0);

    string.skip(3).await.unwrap();
    assert!(string.read(SINGLE).await.is_err());
    assert_eq!(string.len(), None);
  }
}
