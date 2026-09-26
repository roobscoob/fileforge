use core::ops::Deref;

use crate::{
  binary_reader::{
    mutable::Mutable,
    readable::{IntoReadable, Readable},
    snapshot::BinaryReaderSnapshot,
    BinaryReader, MutableMutator,
  },
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    render::{buffer::cell::tag::builtin::report::REPORT_INFO_LINE_TEXT, builtin::text::r#const::ConstText},
    report::Report,
    FileforgeError,
  },
  stream::{error::stream_restore::StreamRestoreError, MutableStream, RestorableStream},
};

pub struct View<'pool, S: RestorableStream<Type = u8>, T: Readable<'pool, S>> {
  start: BinaryReaderSnapshot<'pool, S>,
  reader: BinaryReader<'pool, S>,
  value: T,
}

impl<'pool, S: RestorableStream<Type = u8>, T: Readable<'pool, S>> Deref for View<'pool, S, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.value
  }
}

impl<'pool, S: RestorableStream<Type = u8>, T: Readable<'pool, S>> IntoReadable<'pool, S> for View<'pool, S, T> {
  type Argument = T::Argument;
  type Error = T::Error;

  async fn read(mut reader: BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
    Ok(Self {
      start: reader.snapshot(),
      value: reader.read_with(argument).await?,
      reader,
    })
  }
}

pub enum ViewMutateError<'pool, S: MutableStream<Type = u8> + RestorableStream, T: Mutable<'pool, S> + Readable<'pool, S>> {
  Restore(StreamRestoreError<S::RestoreError>),
  Mutate(<T as Mutable<'pool, S>>::Error),
}

const RETURNING_TO_VIEW: ConstText = ConstText::new("This happened while returning to the start of a value in order to change it.", &REPORT_INFO_LINE_TEXT);

impl<'pool, S: MutableStream<Type = u8> + RestorableStream, T: Mutable<'pool, S> + Readable<'pool, S>> FileforgeError for ViewMutateError<'pool, S, T> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Restore(error) => error.render_into_report(provider, |report| report.with_info_line(&RETURNING_TO_VIEW).apply(callback)),
      Self::Mutate(error) => error.render_into_report(provider, callback),
    }
  }
}

impl<'pool, S: MutableStream<Type = u8> + RestorableStream, T: Mutable<'pool, S> + Readable<'pool, S>> View<'pool, S, T> {
  pub async fn mutate<'l>(&'l mut self) -> Result<T::Mutator<'l>, ViewMutateError<'pool, S, T>> {
    self.reader.restore(self.start.clone()).await.map_err(|e| ViewMutateError::Restore(e))?;
    self.reader.mutate::<T>(&mut self.value).await.map_err(|e| ViewMutateError::Mutate(e))
  }
}
