//! How an archive decides what alignment each file needs.
//!
//! SARC doesn't record alignment anywhere: a file's position is just whatever its writer chose.
//! When an edit moves files, a policy says what each one needs, and the archive keeps it.

use fileforge::{binary_reader::readable::NoneArgument, stream::RestorableStream};

use crate::sead::sarc::{SarcEntry, SarcError, SarcIo};

/// What a policy is told about a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileInfo {
  /// The file's entry, or `None` for a file that's being added and has no entry yet.
  pub entry: Option<SarcEntry>,
  pub hash: u32,
  /// Where the file's data starts, from the start of the archive, or `None` for a file that
  /// doesn't have a place yet.
  pub position: Option<u64>,
}

/// A file a policy is asked about. Besides [`FileInfo`], it can check the file's name, which
/// means reading it, so only ask when it's needed.
pub struct AlignmentQuery<'q, 'pool, S: RestorableStream<Type = u8>> {
  pub(crate) info: FileInfo,
  pub(crate) name: QueryName<'q, 'pool, S>,
}

pub(crate) enum QueryName<'q, 'pool, S: RestorableStream<Type = u8>> {
  /// The name is in the archive.
  Stored(&'q mut SarcIo<'pool, S>),
  /// The file is being added under this name.
  Given(&'q str),
}

impl<'q, 'pool, S: RestorableStream<Type = u8>> AlignmentQuery<'q, 'pool, S> {
  pub fn file(&self) -> &FileInfo {
    &self.info
  }

  /// Whether the file is called `name`.
  pub async fn name_is(&mut self, name: &str) -> Result<bool, SarcError<'pool, S>> {
    match (&mut self.name, &self.info.entry) {
      (QueryName::Given(given), _) => Ok(*given == name),
      (QueryName::Stored(io), Some(entry)) => io.name_is(entry, name).await,
      (QueryName::Stored(_), None) => Ok(false),
    }
  }

  /// Whether the file's name ends with `suffix`, such as `".bntx"`.
  pub async fn name_ends_with(&mut self, suffix: &str) -> Result<bool, SarcError<'pool, S>> {
    match (&mut self.name, &self.info.entry) {
      (QueryName::Given(given), _) => Ok(given.ends_with(suffix)),
      (QueryName::Stored(io), Some(entry)) => io.name_ends_with(entry, suffix).await,
      (QueryName::Stored(_), None) => Ok(false),
    }
  }
}

/// Decides the alignment each file needs. It's chosen when an archive is opened, and every edit
/// follows it.
pub trait AlignmentPolicy {
  /// The alignment `file` needs: a power of two, from the start of the archive.
  async fn alignment<'pool, S: RestorableStream<Type = u8>>(&self, file: &mut AlignmentQuery<'_, 'pool, S>) -> Result<u32, SarcError<'pool, S>>;
}

/// Keeps the alignment each file already has.
///
/// A file's alignment is the largest power of two its position is a multiple of, up to `cap`: a
/// file that happens to sit at 0x10000 doesn't really need 64 KiB. A file without a position yet
/// gets `floor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Preserve {
  pub floor: u32,
  pub cap: u32,
}

impl Default for Preserve {
  fn default() -> Self {
    Preserve { floor: 4, cap: 0x2000 }
  }
}

impl NoneArgument for Preserve {
  fn none() -> Self {
    Preserve::default()
  }
}

impl AlignmentPolicy for Preserve {
  async fn alignment<'pool, S: RestorableStream<Type = u8>>(&self, file: &mut AlignmentQuery<'_, 'pool, S>) -> Result<u32, SarcError<'pool, S>> {
    Ok(match file.info.position {
      None => self.floor,
      Some(0) => self.cap,
      Some(position) => (1u64 << position.trailing_zeros()).min(self.cap as u64) as u32,
    })
  }
}

/// Gives every file the same alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fixed(pub u32);

impl AlignmentPolicy for Fixed {
  async fn alignment<'pool, S: RestorableStream<Type = u8>>(&self, _: &mut AlignmentQuery<'_, 'pool, S>) -> Result<u32, SarcError<'pool, S>> {
    Ok(self.0)
  }
}

/// Gives each file the larger of two policies' alignments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Max<A, B>(pub A, pub B);

impl<A: AlignmentPolicy, B: AlignmentPolicy> AlignmentPolicy for Max<A, B> {
  async fn alignment<'pool, S: RestorableStream<Type = u8>>(&self, file: &mut AlignmentQuery<'_, 'pool, S>) -> Result<u32, SarcError<'pool, S>> {
    Ok(self.0.alignment(file).await?.max(self.1.alignment(file).await?))
  }
}
