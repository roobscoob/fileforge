//! A gallery of example errors.
//!
//! Put `#[story("name", expr)]` on an error type to register an example of it (see
//! `fileforge_macros::story`). Every registered story can then be rendered with
//! [`render_story`], listed with [`iter_stories`] or printed with [`invoke`].
//! Stories only exist when the `story` feature is enabled.

use std::{
  boxed::Box,
  panic::{self, AssertUnwindSafe},
  print, println,
  string::{String, ToString},
  vec,
};

use crate::{
  binary_reader::error::{common::Read, exhausted::ReaderExhaustedError, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
  diagnostic::{
    node::reference::DiagnosticReference,
    pool::{dynamic::DynamicDiagnosticPool, DiagnosticPoolProvider},
    value::DiagnosticValue,
  },
  error::{
    ext::annotations::annotated::{Annotated, AnnotationExt},
    render::{
      buffer::{
        cell::{tag::context::RenderMode, RenderBufferCell},
        RenderBuffer,
      },
      position::RenderPosition,
    },
    report::Report,
    FileforgeError,
  },
  provider, stream,
};

#[doc(hidden)]
pub use inventory;

/// In scope inside every `#[story]` expression.
pub mod prelude {
  pub use super::{read_exhausted, read_failed, StoryProvider, StoryStream, StoryUserError};
  pub use crate::diagnostic::value::DiagnosticValue;
}

/// The size of diagnostic node names when rendering a story.
pub const NODE_NAME_SIZE: usize = 128;

pub struct Story {
  pub name: &'static str,
  /// The name of the type the story is attached to.
  pub type_name: &'static str,
  /// The module the story was declared in, starting with its crate's name.
  pub module: &'static str,
  pub render: fn(RenderMode, usize) -> String,
}

inventory::collect!(Story);

impl Story {
  /// The name of the crate that declared this story.
  pub fn crate_name(&self) -> &'static str {
    self.module.split("::").next().unwrap_or(self.module)
  }
}

pub fn iter_stories() -> impl Iterator<Item = &'static Story> {
  inventory::iter::<Story>.into_iter()
}

/// Renders a story's report as text, one line per rendered row, without trailing whitespace.
///
/// If the error's renderer panics (for example, it is still `todo!()`), returns the panic
/// message instead. The panic hook still runs; see [`quietly`].
pub fn render_story(story: &Story, mode: RenderMode, width: usize) -> Result<String, String> {
  panic::catch_unwind(AssertUnwindSafe(|| (story.render)(mode, width))).map_err(|payload| {
    payload
      .downcast_ref::<&str>()
      .map(|message| message.to_string())
      .or_else(|| payload.downcast_ref::<String>().cloned())
      .unwrap_or_else(|| "<non-string panic payload>".to_string())
  })
}

/// Runs `f` with the panic hook silenced, so renderers that panic don't print to stderr.
pub fn quietly<T>(f: impl FnOnce() -> T) -> T {
  let hook = panic::take_hook();
  panic::set_hook(Box::new(|_| {}));
  let result = f();
  panic::set_hook(hook);
  result
}

/// Prints every story to stdout.
pub fn invoke() {
  quietly(|| {
    for story in iter_stories() {
      println!("{} / {} ({})", story.type_name, story.name, story.module);

      match render_story(story, RenderMode::TerminalAnsi, 80) {
        Ok(rendered) => print!("{rendered}"),
        Err(message) => println!("PANICKED: {message}"),
      }

      println!();
    }
  });
}

#[doc(hidden)]
pub fn render<E: FileforgeError>(error: &E, pool: &DynamicDiagnosticPool, mode: RenderMode, width: usize) -> String {
  let mut out = String::new();

  error.render_into_report::<_, NODE_NAME_SIZE>(pool, |report| {
    // The report's height is only known after rendering it, so grow the buffer until it fits.
    let mut lines = 64;

    loop {
      let mut cells = vec![RenderBufferCell::default(); width * lines];
      let mut buffer = RenderBuffer::new(&mut cells, width, 0);
      let height = buffer.canvas_at(RenderPosition::zero()).write(&report).unwrap().get_line_height();

      if height > lines {
        lines = height;
        continue;
      }

      let mut rendered = String::new();
      buffer.flush_into(&mut rendered, mode).unwrap();

      for line in rendered.lines().take(height) {
        out.push_str(line.trim_end());
        out.push('\n');
      }

      break;
    }
  });

  out
}

/// A stand-in for an error from outside the library (a stream, provider or user callback), for
/// stories of variants that wrap one.
#[derive(Debug)]
pub struct StoryUserError;

impl FileforgeError for StoryUserError {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    Report::new::<Self>(provider, &"Example error from outside the library").apply(callback)
  }
}

impl stream::error::user_mutate::UserMutateError for StoryUserError {}
impl stream::error::user_overwrite::UserOverwriteError for StoryUserError {}
impl stream::error::user_partition::UserPartitionError for StoryUserError {}
impl stream::error::user_read::UserReadError for StoryUserError {}
impl stream::error::user_restore::UserRestoreError for StoryUserError {}
impl stream::error::user_rewind::UserRewindError for StoryUserError {}
impl stream::error::user_seek::UserSeekError for StoryUserError {}
impl stream::error::user_skip::UserSkipError for StoryUserError {}
impl stream::error::user_write::UserWriteError for StoryUserError {}
impl provider::error::user_mutate::UserMutateError for StoryUserError {}
impl provider::error::user_partition::UserPartitionError for StoryUserError {}
impl provider::error::user_read::UserReadError for StoryUserError {}
impl provider::error::user_resize::UserResizeError for StoryUserError {}
impl provider::error::user_slice::UserSliceError for StoryUserError {}

/// A stand-in stream for stories of types that are generic over one. Every error it could
/// produce is a [`StoryUserError`]. It is never read from; its methods panic.
#[derive(Debug)]
pub struct StoryStream;

impl stream::ReadableStream for StoryStream {
  type Type = u8;
  type ReadError = StoryUserError;
  type SkipError = StoryUserError;

  fn offset(&self) -> u64 {
    unimplemented!("StoryStream only exists to name types in stories")
  }

  async fn read<const SIZE: usize, V>(&mut self, _: impl AsyncFnOnce(&[u8; SIZE]) -> V) -> Result<V, stream::StreamReadError<StoryUserError>> {
    unimplemented!("StoryStream only exists to name types in stories")
  }

  async fn skip(&mut self, _: u64) -> Result<(), stream::StreamSkipError<StoryUserError>> {
    unimplemented!("StoryStream only exists to name types in stories")
  }
}

impl stream::RestorableStream for StoryStream {
  type Snapshot = ();
  type RestoreError = StoryUserError;

  fn snapshot(&self) -> Self::Snapshot {}

  async fn restore(&mut self, _: Self::Snapshot) -> Result<(), stream::error::stream_restore::StreamRestoreError<StoryUserError>> {
    unimplemented!("StoryStream only exists to name types in stories")
  }
}

impl stream::MutableStream for StoryStream {
  type MutateError = StoryUserError;

  async fn mutate<const SIZE: usize, V: crate::control_flow::ControlFlow>(&mut self, _: impl AsyncFnOnce(&mut [u8; SIZE]) -> V) -> Result<V, stream::StreamMutateError<StoryUserError>> {
    unimplemented!("StoryStream only exists to name types in stories")
  }
}

impl stream::ResizableStream for StoryStream {
  type OverwriteError = StoryUserError;

  async fn overwrite<const SIZE: usize>(&mut self, _: u64, _: [u8; SIZE]) -> Result<(), stream::error::stream_overwrite::StreamOverwriteError<StoryUserError>> {
    unimplemented!("StoryStream only exists to name types in stories")
  }
}

/// A stand-in provider for stories of types that are generic over one. Every error it could
/// produce is a [`StoryUserError`]. It is never read from; its methods panic.
#[derive(Debug)]
pub struct StoryProvider;

impl provider::Provider for StoryProvider {
  type Type = u8;

  type StaticSliceProvider<'l, const SIZE: usize> = StoryProvider;
  type DynamicSliceProvider<'l> = StoryProvider;

  type SliceError = StoryUserError;
  type ReadError = StoryUserError;

  fn len(&self) -> u64 {
    unimplemented!("StoryProvider only exists to name types in stories")
  }

  fn slice<'l, const SIZE: usize>(&'l self, _: u64) -> Result<StoryProvider, provider::error::provider_slice::ProviderSliceError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }

  fn slice_dynamic<'l>(&'l self, _: u64, _: Option<u64>) -> Result<StoryProvider, provider::error::provider_slice::ProviderSliceError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }

  async fn read<const SIZE: usize, V>(&self, _: u64, _: provider::hint::ReadHint, _: impl for<'v> AsyncFnOnce(&'v [u8; SIZE]) -> V) -> Result<V, provider::error::provider_read::ProviderReadError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }
}

impl provider::MutProvider for StoryProvider {
  type MutateError = StoryUserError;

  type StaticMutSliceProvider<'l, const SIZE: usize> = StoryProvider;
  type DynamicMutSliceProvider<'l> = StoryProvider;

  async fn mutate<const SIZE: usize, V>(&mut self, _: u64, _: impl for<'v> AsyncFnOnce(&'v mut [u8; SIZE]) -> V) -> Result<V, provider::error::provider_mutate::ProviderMutateError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }

  fn mut_slice<'l, const SIZE: usize>(&'l mut self, _: u64) -> Result<StoryProvider, provider::error::provider_slice::ProviderSliceError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }

  fn mut_slice_dynamic<'l>(&'l mut self, _: u64, _: Option<u64>) -> Result<StoryProvider, provider::error::provider_slice::ProviderSliceError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }
}

impl provider::ResizableProvider for StoryProvider {
  type ResizeError = StoryUserError;

  async fn resize_at(&mut self, _: u64, _: u64, _: u64) -> Result<(), provider::error::provider_resize::ProviderResizeError<StoryUserError>> {
    unimplemented!("StoryProvider only exists to name types in stories")
  }
}

/// A failed read of a `T`, where the data ran out: `T` was read at `offset`, but the stream held
/// only `stream_length` bytes. This is the error most wrappers hold.
pub fn read_exhausted<'pool, T, U: stream::UserReadError>(
  container: Option<DiagnosticReference<'pool>>,
  offset: u64,
  stream_length: DiagnosticValue<'pool, u64>,
) -> Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>> {
  let error = GetPrimitiveError::ReaderExhausted(ReaderExhaustedError {
    container,
    length: DiagnosticValue(core::mem::size_of::<T>() as u64, None),
    offset,
    stream_length,
    t: Read,
  });

  Err::<(), _>(error).annotate(PrimitiveName::for_type::<T>()).unwrap_err()
}

/// A failed read of a `T`, where the stream itself failed with `error`.
pub fn read_failed<'pool, T, U: stream::UserReadError>(error: U) -> Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, U>> {
  Err::<(), _>(GetPrimitiveError::User(error)).annotate(PrimitiveName::for_type::<T>()).unwrap_err()
}
