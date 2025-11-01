use core::{fmt::Debug, future::ready};

use crate::{
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    FileforgeError,
    render::buffer::cell::tag::builtin::report::{REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT},
    report::{Report, kind::ReportKind},
  },
  provider::{
    MutProvider, Provider, ResizableProvider,
    error::{provider_mutate::ProviderMutateError, provider_read::ProviderReadError, provider_resize::ProviderResizeError, provider_slice::ProviderSliceError},
    hint::ReadHint,
  },
  stream::{
    DynamicPartitionableStream, MutableStream, ReadableStream, ResizableStream, RestorableStream, RewindableStream, SeekableStream, StaticPartitionableStream,
    error::{
      stream_exhausted::StreamExhaustedError, stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_partition::StreamPartitionError, stream_read::StreamReadError,
      stream_rewind::StreamRewindError, stream_seek::StreamSeekError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError, user_mutate::UserMutateError,
      user_overwrite::UserOverwriteError, user_partition::UserPartitionError, user_read::UserReadError, user_rewind::UserRewindError, user_seek::UserSeekError, user_skip::UserSkipError,
    },
  },
};

pub struct ProviderStream<P: Provider> {
  poisoned: bool,
  hint: ReadHint,
  offset: u64,
  provider: P,
}

impl<P: Provider> ProviderStream<P> {
  pub fn new(provider: P, hint: ReadHint) -> Self {
    Self {
      provider,
      hint,
      offset: 0,
      poisoned: false,
    }
  }

  pub fn set_read_hint(&mut self, hint: ReadHint) {
    self.hint = hint
  }

  fn assert_not_poisoned(&self) -> Result<(), ProviderStreamPoisonedError> {
    if self.poisoned { Err(ProviderStreamPoisonedError) } else { Ok(()) }
  }
}

impl<P: Provider> ReadableStream for ProviderStream<P>
where
  P::ReadError: UserReadError,
{
  type Type = P::Type;
  type ReadError = ProviderStreamError<P::ReadError>;

  fn len(&self) -> Option<u64> {
    Some(self.provider.len())
  }
  fn offset(&self) -> u64 {
    self.offset
  }
  fn remaining(&self) -> Option<u64> {
    Some(self.provider.len() - self.offset)
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[P::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    self.assert_not_poisoned().map_err(|e| StreamReadError::User(ProviderStreamError::Poisoned(e)))?;

    return match self.provider.read(self.offset, self.hint, reader).await {
      Ok(v) => {
        self.offset += SIZE as u64;

        Ok(v)
      }

      Err(ProviderReadError::User(u)) => Err(StreamReadError::User(ProviderStreamError::Specific(u))),

      Err(ProviderReadError::OutOfBounds(oob)) => Err(StreamReadError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).expect("Read length should be non-None"))),
    };
  }

  type SkipError = ProviderStreamPoisonedError;

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    self.assert_not_poisoned().map_err(|e| StreamSkipError::User(e))?;

    let seek_point = StreamSkipError::assert_relative_forwards(self.provider.len(), self.offset, size)?;

    self.offset = seek_point;
    Ok(())
  }
}

impl<P: Provider> RewindableStream for ProviderStream<P>
where
  <P as Provider>::ReadError: UserReadError,
{
  type RewindError = ProviderStreamPoisonedError;

  async fn rewind(&mut self, size: u64) -> Result<(), StreamRewindError<Self::RewindError>> {
    self.assert_not_poisoned().map_err(|e| StreamRewindError::User(e))?;

    let seek_point = StreamRewindError::assert_relative_backwards(self.provider.len(), self.offset, size)?;

    self.offset = seek_point;
    Ok(())
  }
}

impl<P: Provider> SeekableStream for ProviderStream<P>
where
  <P as Provider>::ReadError: UserReadError,
{
  type SeekError = ProviderStreamPoisonedError;

  async fn seek(&mut self, offset: u64) -> Result<(), StreamSeekError<Self::SeekError>> {
    self.assert_not_poisoned().map_err(|e| StreamSeekError::User(e))?;

    StreamSeekOutOfBoundsError::assert(self.provider.len(), offset)?;

    self.offset = offset;
    Ok(())
  }
}

impl<P: MutProvider> MutableStream for ProviderStream<P>
where
  <P as Provider>::ReadError: UserReadError,
  <P as MutProvider>::MutateError: UserMutateError,
{
  type MutateError = ProviderStreamError<P::MutateError>;

  async fn mutate<const SIZE: usize, V>(&mut self, mutator: impl AsyncFnOnce(&mut [P::Type; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>> {
    self.assert_not_poisoned().map_err(|e| StreamMutateError::User(ProviderStreamError::Poisoned(e)))?;

    return match self.provider.mutate(self.offset, mutator).await {
      Ok(v) => {
        self.offset += SIZE as u64;

        Ok(v)
      }

      Err(ProviderMutateError::User(u)) => Err(StreamMutateError::User(ProviderStreamError::Specific(u))),

      Err(ProviderMutateError::OutOfBounds(oob)) => Err(StreamMutateError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).expect("Read length should be non-None"))),
    };
  }
}

impl<P: ResizableProvider> ResizableStream for ProviderStream<P>
where
  <P as Provider>::ReadError: UserReadError,
{
  type OverwriteError = ProviderOverwriteError<P>;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: [P::Type; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    if self.poisoned {
      return Err(StreamOverwriteError::User(ProviderOverwriteError::Poisoned(ProviderStreamPoisonedError)));
    }

    match self.provider.resize_at(self.offset, length, SIZE as u64).await {
      Ok(_) => {}

      Err(ProviderResizeError::OutOfBounds(oob)) => {
        return Err(StreamOverwriteError::StreamExhausted(
          Option::<StreamExhaustedError>::from(oob).expect("Read length should be non-None"),
        ));
      }

      Err(ProviderResizeError::User(u)) => return Err(StreamOverwriteError::User(ProviderOverwriteError::Allocate(u))),
    };

    match self.provider.mutate(self.offset, |write_target: &mut [P::Type; SIZE]| ready(*write_target = data)).await {
      Ok(_) => {}

      Err(ProviderMutateError::OutOfBounds(_)) => unreachable!("Bounds checked in previous call"),

      Err(ProviderMutateError::User(u)) => return Err(StreamOverwriteError::User(ProviderOverwriteError::Write(u))),
    };

    Ok(())
  }
}

impl<'l, const SIZE: usize, P: Provider + 'l> StaticPartitionableStream<'l, SIZE> for ProviderStream<P>
where
  P::SliceError: UserPartitionError,
  P::ReadError: UserReadError,
  <P::StaticSliceProvider<'l, SIZE> as Provider>::ReadError: UserReadError,
  P::StaticSliceProvider<'l, SIZE>: 'l,
{
  type PartitionError = P::SliceError;

  type Partition = ProviderStream<P::StaticSliceProvider<'l, SIZE>>;

  async fn partition(&'l mut self) -> Result<Self::Partition, StreamPartitionError<Self::PartitionError>> {
    match self.provider.slice::<SIZE>(self.offset) {
      Ok(provider) => Ok(ProviderStream::new(provider, self.hint)),
      Err(ProviderSliceError::User(u)) => Err(StreamPartitionError::User(u)),
      Err(ProviderSliceError::OutOfBounds(oob)) => Err(StreamPartitionError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).unwrap())),
    }
  }
}

impl<'l, P: Provider + 'l> DynamicPartitionableStream<'l> for ProviderStream<P>
where
  P::SliceError: UserPartitionError,
  P::ReadError: UserReadError,
  <P::DynamicSliceProvider<'l> as Provider>::ReadError: UserReadError,
  P::DynamicSliceProvider<'l>: 'l,
{
  type PartitionError = P::SliceError;

  type PartitionDynamic = ProviderStream<P::DynamicSliceProvider<'l>>;

  async fn partition_dynamic(&'l mut self, size: u64) -> Result<Self::PartitionDynamic, StreamPartitionError<Self::PartitionError>> {
    match self.provider.slice_dynamic(self.offset, Some(size)) {
      Ok(provider) => Ok(ProviderStream::new(provider, self.hint)),
      Err(ProviderSliceError::User(u)) => Err(StreamPartitionError::User(u)),
      Err(ProviderSliceError::OutOfBounds(oob)) => Err(StreamPartitionError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).unwrap())),
    }
  }
}

pub enum ProviderOverwriteError<P: ResizableProvider> {
  Poisoned(ProviderStreamPoisonedError),
  Allocate(P::ResizeError),
  Write(P::MutateError),
}

impl<'pool, P: ResizableProvider> FileforgeError for ProviderOverwriteError<P> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, Pr: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref Pr,
    callback: impl for<'tag, 'b, 'p2> FnMut(Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, Pr>) -> (),
  ) {
    match self {
      Self::Poisoned(p) => p.render_into_report(provider, callback),
      Self::Allocate(a) => a.render_into_report(provider, callback),
      Self::Write(w) => w.render_into_report(provider, callback),
    }
  }
}

impl<P: ResizableProvider> UserOverwriteError for ProviderOverwriteError<P> {}

#[derive(Debug)]
pub enum ProviderStreamError<T: FileforgeError> {
  Poisoned(ProviderStreamPoisonedError),
  Specific(T),
}

impl<T: FileforgeError> From<T> for ProviderStreamError<T> {
  fn from(value: T) -> Self {
    Self::Specific(value)
  }
}

impl<T: UserReadError> UserReadError for ProviderStreamError<T> {}
impl<T: UserMutateError> UserMutateError for ProviderStreamError<T> {}

impl<T: FileforgeError> FileforgeError for ProviderStreamError<T> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    callback: impl for<'tag, 'b, 'pool> FnMut(Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    match self {
      Self::Poisoned(p) => p.render_into_report(provider, callback),
      Self::Specific(s) => s.render_into_report(provider, callback),
    }
  }
}

#[derive(Debug)]
pub struct ProviderStreamPoisonedError;

impl UserSkipError for ProviderStreamPoisonedError {}
impl UserSeekError for ProviderStreamPoisonedError {}
impl UserRewindError for ProviderStreamPoisonedError {}

impl FileforgeError for ProviderStreamPoisonedError {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: DiagnosticPoolProvider>(
    &self,
    provider: &'pool_ref P,
    mut callback: impl for<'tag, 'b, 'pool> FnMut(Report<'tag, 'b, 'pool, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    let report = Report::new::<Self>(provider, ReportKind::Error, "Provider Stream Poisoned")
      .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "This is a low-level error, intended to be consumed by higher-level error handling code. This error is not intended to be displayed to the user. If you're seeing this error and *not* a library author, it may be confusing. Please report this error to the library author."))
      .unwrap()
      .with_info_line(const_text!([&REPORT_INFO_LINE_TEXT] "This error occurs when a ProviderStream is \"poisoned\".")).unwrap()
      .with_info_line(const_text!([&REPORT_INFO_LINE_TEXT] "Poisoning occurs when a provider encounters an error from the stream it cannot recover from.")).unwrap()
      .with_info_line(const_text!([&REPORT_INFO_LINE_TEXT] "If this error is being displayed, you're probably not properly handling a previous error emitted by the ProviderStream")).unwrap();

    callback(report)
  }
}
