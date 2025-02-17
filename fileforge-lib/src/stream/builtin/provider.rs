use core::future::ready;

use crate::{
  error::{
    render::buffer::cell::tag::builtin::report::{REPORT_FLAG_LINE_TEXT, REPORT_INFO_LINE_TEXT},
    report::{kind::ReportKind, Report},
    FileforgeError,
  },
  provider::{
    error::{provider_mutate::ProviderMutateError, provider_read::ProviderReadError, provider_resize::ProviderResizeError, provider_slice::ProviderSliceError},
    hint::ReadHint,
    MutProvider, Provider, ResizableProvider,
  },
  stream::{
    error::{
      stream_exhausted::StreamExhaustedError, stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_partition::StreamPartitionError, stream_read::StreamReadError,
      stream_rewind::StreamRewindError, stream_seek::StreamSeekError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError, user_mutate::UserMutateError,
      user_overwrite::UserOverwriteError, user_partition::UserPartitionError, user_read::UserReadError, user_rewind::UserRewindError, user_seek::UserSeekError, user_skip::UserSkipError,
    },
    DynamicPartitionableStream, MutableStream, ReadableStream, ResizableStream, RewindableStream, SeekableStream, SkippableStream, StaticPartitionableStream,
  },
};

pub struct ProviderStream<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> {
  poisoned: bool,
  hint: ReadHint,
  offset: u64,
  provider: P,
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> ProviderStream<NODE_NAME_SIZE, P> {
  pub fn new(provider: P, hint: ReadHint) -> Self {
    Self {
      provider,
      hint,
      offset: 0,
      poisoned: false,
    }
  }

  pub fn set_read_hint(&mut self, hint: ReadHint) { self.hint = hint }

  fn assert_not_poisoned(&self) -> Result<(), ProviderStreamPoisonedError> {
    if self.poisoned {
      Err(ProviderStreamPoisonedError)
    } else {
      Ok(())
    }
  }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> ReadableStream<NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  P::ReadError: UserReadError<NODE_NAME_SIZE>,
{
  type ReadError = ProviderStreamError<NODE_NAME_SIZE, P::ReadError>;

  fn len(&self) -> Option<u64> { Some(self.provider.len()) }
  fn offset(&self) -> u64 { self.offset }
  fn remaining(&self) -> Option<u64> { Some(self.provider.len() - self.offset) }

  async fn read<const SIZE: usize, V, R: core::future::Future<Output = V>>(&mut self, reader: impl FnOnce(&[u8; SIZE]) -> R) -> Result<V, StreamReadError<NODE_NAME_SIZE, Self::ReadError>> {
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
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> SkippableStream<NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
{
  type SkipError = ProviderStreamPoisonedError;

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<NODE_NAME_SIZE, Self::SkipError>> {
    self.assert_not_poisoned().map_err(|e| StreamSkipError::User(e))?;

    let seek_point = StreamSkipError::assert_relative_forwards(self.provider.len(), self.offset, size)?;

    self.offset = seek_point;
    Ok(())
  }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> RewindableStream<NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
{
  type RewindError = ProviderStreamPoisonedError;

  async fn rewind(&mut self, size: u64) -> Result<(), StreamRewindError<NODE_NAME_SIZE, Self::RewindError>> {
    self.assert_not_poisoned().map_err(|e| StreamRewindError::User(e))?;

    let seek_point = StreamRewindError::assert_relative_backwards(self.provider.len(), self.offset, size)?;

    self.offset = seek_point;
    Ok(())
  }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> SeekableStream<NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
{
  type SeekError = ProviderStreamPoisonedError;

  async fn seek(&mut self, offset: u64) -> Result<(), StreamSeekError<NODE_NAME_SIZE, Self::SeekError>> {
    self.assert_not_poisoned().map_err(|e| StreamSeekError::User(e))?;

    StreamSeekOutOfBoundsError::assert(self.provider.len(), offset)?;

    self.offset = offset;
    Ok(())
  }
}

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> MutableStream<NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
  P: MutProvider<NODE_NAME_SIZE>,
  <P as MutProvider<NODE_NAME_SIZE>>::MutateError: UserMutateError<NODE_NAME_SIZE>,
{
  type MutateError = ProviderStreamError<NODE_NAME_SIZE, P::MutateError>;

  async fn mutate<const SIZE: usize, V, R: core::future::Future<Output = V>>(&mut self, mutator: impl FnOnce(&mut [u8; SIZE]) -> R) -> Result<V, StreamMutateError<NODE_NAME_SIZE, Self::MutateError>> {
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

impl<const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> ResizableStream<NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
  P: ResizableProvider<NODE_NAME_SIZE>,
{
  type OverwriteError = ProviderOverwriteError<NODE_NAME_SIZE, P>;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: &[u8; SIZE]) -> Result<(), StreamOverwriteError<NODE_NAME_SIZE, Self::OverwriteError>> {
    if self.poisoned {
      return Err(StreamOverwriteError::User(ProviderOverwriteError::Poisoned(ProviderStreamPoisonedError)));
    }

    match self.provider.resize_at(self.offset, length, SIZE as u64).await {
      Ok(_) => {}

      Err(ProviderResizeError::OutOfBounds(oob)) => {
        return Err(StreamOverwriteError::StreamExhausted(
          Option::<StreamExhaustedError>::from(oob).expect("Read length should be non-None"),
        ))
      }

      Err(ProviderResizeError::User(u)) => return Err(StreamOverwriteError::User(ProviderOverwriteError::Allocate(u))),
    };

    match self.provider.mutate(self.offset, |write_target: &mut [u8; SIZE]| ready(write_target.copy_from_slice(data))).await {
      Ok(_) => {}

      Err(ProviderMutateError::OutOfBounds(_)) => unreachable!("Bounds checked in previous call"),

      Err(ProviderMutateError::User(u)) => return Err(StreamOverwriteError::User(ProviderOverwriteError::Write(u))),
    };

    Ok(())
  }
}

impl<'l, const SIZE: usize, const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> StaticPartitionableStream<'l, NODE_NAME_SIZE, SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::SliceError: UserPartitionError<NODE_NAME_SIZE>,
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
  <<P as Provider<NODE_NAME_SIZE>>::StaticSliceProvider<'l, SIZE> as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
  <P as Provider<NODE_NAME_SIZE>>::StaticSliceProvider<'l, SIZE>: 'l,
  P: 'l,
{
  type PartitionError = P::SliceError;

  type Partition = ProviderStream<NODE_NAME_SIZE, P::StaticSliceProvider<'l, SIZE>>;

  async fn partition(&'l mut self) -> Result<Self::Partition, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>> {
    match self.provider.slice::<SIZE>(self.offset) {
      Ok(provider) => Ok(ProviderStream::new(provider, self.hint)),
      Err(ProviderSliceError::User(u)) => Err(StreamPartitionError::User(u)),
      Err(ProviderSliceError::OutOfBounds(oob)) => Err(StreamPartitionError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).unwrap())),
    }
  }
}

impl<'l, const NODE_NAME_SIZE: usize, P: Provider<NODE_NAME_SIZE>> DynamicPartitionableStream<'l, NODE_NAME_SIZE> for ProviderStream<NODE_NAME_SIZE, P>
where
  <P as Provider<NODE_NAME_SIZE>>::SliceError: UserPartitionError<NODE_NAME_SIZE>,
  <P as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
  <<P as Provider<NODE_NAME_SIZE>>::DynamicSliceProvider<'l> as Provider<NODE_NAME_SIZE>>::ReadError: UserReadError<NODE_NAME_SIZE>,
  <P as Provider<NODE_NAME_SIZE>>::DynamicSliceProvider<'l>: 'l,
  P: 'l,
{
  type PartitionError = P::SliceError;

  type PartitionDynamic = ProviderStream<NODE_NAME_SIZE, P::DynamicSliceProvider<'l>>;

  async fn partition_dynamic(&'l mut self, size: u64) -> Result<Self::PartitionDynamic, StreamPartitionError<NODE_NAME_SIZE, Self::PartitionError>> {
    match self.provider.slice_dynamic(self.offset, Some(size)) {
      Ok(provider) => Ok(ProviderStream::new(provider, self.hint)),
      Err(ProviderSliceError::User(u)) => Err(StreamPartitionError::User(u)),
      Err(ProviderSliceError::OutOfBounds(oob)) => Err(StreamPartitionError::StreamExhausted(Option::<StreamExhaustedError>::from(oob).unwrap())),
    }
  }
}

pub enum ProviderOverwriteError<const NODE_NAME_SIZE: usize, P: ResizableProvider<NODE_NAME_SIZE>> {
  Poisoned(ProviderStreamPoisonedError),
  Allocate(P::ResizeError),
  Write(P::MutateError),
}

impl<const NODE_NAME_SIZE: usize, P: ResizableProvider<NODE_NAME_SIZE>> FileforgeError<NODE_NAME_SIZE> for ProviderOverwriteError<NODE_NAME_SIZE, P> {
  fn render_into_report(&self, callback: impl FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> ()) {
    match self {
      Self::Poisoned(p) => p.render_into_report(callback),
      Self::Allocate(a) => a.render_into_report(callback),
      Self::Write(w) => w.render_into_report(callback),
    }
  }
}

impl<const NODE_NAME_SIZE: usize, P: ResizableProvider<NODE_NAME_SIZE>> UserOverwriteError<NODE_NAME_SIZE> for ProviderOverwriteError<NODE_NAME_SIZE, P> {}

pub enum ProviderStreamError<const NODE_NAME_SIZE: usize, T: FileforgeError<NODE_NAME_SIZE>> {
  Poisoned(ProviderStreamPoisonedError),
  Specific(T),
}

impl<const NODE_NAME_SIZE: usize, T: FileforgeError<NODE_NAME_SIZE>> From<T> for ProviderStreamError<NODE_NAME_SIZE, T> {
  fn from(value: T) -> Self { Self::Specific(value) }
}

impl<const NODE_NAME_SIZE: usize, T: UserReadError<NODE_NAME_SIZE>> UserReadError<NODE_NAME_SIZE> for ProviderStreamError<NODE_NAME_SIZE, T> {}
impl<const NODE_NAME_SIZE: usize, T: UserMutateError<NODE_NAME_SIZE>> UserMutateError<NODE_NAME_SIZE> for ProviderStreamError<NODE_NAME_SIZE, T> {}

impl<const NODE_NAME_SIZE: usize, T: FileforgeError<NODE_NAME_SIZE>> FileforgeError<NODE_NAME_SIZE> for ProviderStreamError<NODE_NAME_SIZE, T> {
  fn render_into_report(&self, callback: impl FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> ()) {
    match self {
      Self::Poisoned(p) => p.render_into_report(callback),
      Self::Specific(s) => s.render_into_report(callback),
    }
  }
}

pub struct ProviderStreamPoisonedError;

impl<const NODE_NAME_SIZE: usize> UserSkipError<NODE_NAME_SIZE> for ProviderStreamPoisonedError {}
impl<const NODE_NAME_SIZE: usize> UserSeekError<NODE_NAME_SIZE> for ProviderStreamPoisonedError {}
impl<const NODE_NAME_SIZE: usize> UserRewindError<NODE_NAME_SIZE> for ProviderStreamPoisonedError {}

impl<const NODE_NAME_SIZE: usize> FileforgeError<NODE_NAME_SIZE> for ProviderStreamPoisonedError {
  fn render_into_report(&self, mut callback: impl FnMut(crate::error::report::Report<NODE_NAME_SIZE>) -> ()) {
    let report = Report::new::<Self>(ReportKind::Error, "Provider Stream Poisoned")
      .with_flag_line(const_text!([&REPORT_FLAG_LINE_TEXT] "This is a low-level error, intended to be consumed by higher-level error handling code. This error is not intended to be displayed to the user. If you're seeing this error and *not* a library author, it may be confusing. Please report this error to the library author."))
      .unwrap()
      .with_info_line(const_text!([&REPORT_INFO_LINE_TEXT] "This error occurs when a ProviderStream is \"poisoned\".")).unwrap()
      .with_info_line(const_text!([&REPORT_INFO_LINE_TEXT] "Poisoning occurs when a provider encounters an error from the stream it cannot recover from.")).unwrap()
      .with_info_line(const_text!([&REPORT_INFO_LINE_TEXT] "If this error is being displayed, you're probably not properly handling a previous error emitted by the ProviderStream")).unwrap();

    callback(report)
  }
}
