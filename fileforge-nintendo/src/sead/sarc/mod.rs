pub mod align;
pub mod edit;
pub mod error;
pub mod file;
pub mod header;
pub mod readable;
pub mod sfat;
pub mod validate;

use core::hash::Hasher;

use fileforge::{
  binary_reader::{snapshot::BinaryReaderSnapshot, BinaryReader, PrimitiveReader},
  provider::hint::ReadHint,
  stream::{
    builtin::{provider::ProviderStream, read_until::ReadUntil},
    error::stream_cmp::StreamCmpError,
    extensions::readable::ReadableStreamExt,
    RestorableStream, StreamReadError,
  },
};

pub use align::{AlignmentPolicy, AlignmentQuery, FileInfo, Fixed, Max, Preserve};
pub use error::SarcError;
pub use file::SarcFile;
pub use sfat::name_table::hasher::HashMode;

use crate::sead::sarc::{
  header::{SarcHeader, SARC_HEADER_SIZE},
  sfat::{
    entry::{attributes::FilenameAttributes, SfatEntry, SFAT_ENTRY_SIZE},
    header::{SfatHeader, SFAT_HEADER_SIZE},
    name_table::{hasher::SfntHasher, header::SFNT_HEADER_SIZE},
  },
};

/// A SARC archive, read from a stream it owns.
///
/// Opening one reads the SARC, SFAT and SFNT headers; everything else is read when it's asked
/// for. The archive moves around its stream, so the stream must be restorable: going back to an
/// earlier part restores a snapshot and skips forward from there.
///
/// `A` decides the alignment each file needs when the archive is edited. See [`AlignmentPolicy`].
pub struct Sarc<'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy = Preserve> {
  pub(crate) io: SarcIo<'pool, S>,
  pub(crate) policy: A,
}

/// One entry of a SARC archive: a file's hash, name and place.
///
/// Entries are read fresh each time; one read before an edit may no longer match the archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SarcEntry {
  /// Its position in the SFAT, which is sorted by hash.
  pub index: u32,
  pub hash: u32,
  /// Where its name is, if it has one.
  pub name: Option<FilenameAttributes>,
  /// Where its data starts, from the start of the data section.
  pub start: u32,
  /// Where its data ends, from the start of the data section.
  pub end: u32,
}

impl SarcEntry {
  /// The length of the file's data.
  pub fn len(&self) -> u64 {
    (self.end - self.start) as u64
  }
}

/// Everything about the archive but its alignment policy, so the policy can be asked about a file
/// while it reads from the archive.
pub(crate) struct SarcIo<'pool, S: RestorableStream<Type = u8>> {
  pub(crate) reader: BinaryReader<'pool, S>,
  /// Where the archive starts in the reader.
  pub(crate) origin: u64,
  /// A snapshot at the start of the archive, which stays valid through edits.
  pub(crate) start: BinaryReaderSnapshot<'pool, S>,
  /// A snapshot at the start of the SFAT's entries, to return to them without starting over.
  /// Edits can invalidate it (over Yaz0, for one), so they clear it; it's taken again when needed.
  pub(crate) entries: Option<BinaryReaderSnapshot<'pool, S>>,
  pub(crate) header: SarcHeader,
  pub(crate) sfat: SfatHeader,
  pub(crate) hash_mode: HashMode,
}

impl<'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy> Sarc<'pool, S, A> {
  pub fn header(&self) -> &SarcHeader {
    &self.io.header
  }

  pub fn file_count(&self) -> u16 {
    self.io.sfat.file_count
  }

  pub fn hash_multiplier(&self) -> u32 {
    self.io.sfat.hash_multiplier
  }

  /// Where the SFAT's entries start, from the start of the archive.
  pub fn entries_offset(&self) -> u64 {
    ENTRIES_OFFSET
  }

  /// Where the file names start, from the start of the archive.
  pub fn names_offset(&self) -> u64 {
    self.io.names_offset()
  }

  /// Where the file data starts, from the start of the archive.
  pub fn data_offset(&self) -> u64 {
    self.io.data_offset()
  }

  pub fn policy(&self) -> &A {
    &self.policy
  }

  pub fn hash_mode(&self) -> HashMode {
    self.io.hash_mode
  }

  /// Chooses how names are hashed. Modern archives use [`HashMode::Signed`], the default; some
  /// Wii U archives use [`HashMode::Unsigned`].
  pub fn set_hash_mode(&mut self, mode: HashMode) {
    self.io.hash_mode = mode;
  }

  /// The hash of `name` in this archive.
  pub fn hash_name(&self, name: &str) -> u32 {
    self.io.hash_name(name)
  }

  /// Reads entry `index`, checking that its name and data are inside the archive.
  pub async fn entry(&mut self, index: u32) -> Result<SarcEntry, SarcError<'pool, S>> {
    self.io.entry(index).await
  }

  /// Finds the file called `name`, the way sead does.
  pub async fn find(&mut self, name: &str) -> Result<Option<SarcEntry>, SarcError<'pool, S>> {
    self.io.find(name).await
  }

  /// Whether `entry` is called `name`. An entry without a name is called nothing.
  pub async fn name_is(&mut self, entry: &SarcEntry, name: &str) -> Result<bool, SarcError<'pool, S>> {
    self.io.name_is(entry, name).await
  }

  /// The length of `entry`'s name, if it has one.
  pub async fn name_len(&mut self, entry: &SarcEntry) -> Result<Option<u64>, SarcError<'pool, S>> {
    self.io.name_len(entry).await
  }

  /// `entry`'s name as a stream, which ends at its terminator, if it has one.
  pub async fn name(&mut self, entry: &SarcEntry) -> Result<Option<ReadUntil<&mut S>>, SarcError<'pool, S>> {
    let Some(position) = self.io.name_position(entry) else { return Ok(None) };

    self.io.seek(position).await?;
    Ok(Some(ReadUntil::new(self.io.reader.stream_mut(), 0)))
  }

  /// `entry`'s data, as a stream.
  pub async fn file(&mut self, entry: &SarcEntry) -> Result<SarcFile<'_, 'pool, S, A>, SarcError<'pool, S>> {
    SarcFile::open(self, entry.index).await
  }
}

/// Where the SFAT's entries start, from the start of the archive.
pub(crate) const ENTRIES_OFFSET: u64 = (SARC_HEADER_SIZE + SFAT_HEADER_SIZE) as u64;

/// Where the file names start in an archive of `file_count` files.
pub(crate) fn names_offset(file_count: u16) -> u64 {
  ENTRIES_OFFSET + file_count as u64 * SFAT_ENTRY_SIZE + SFNT_HEADER_SIZE as u64
}

impl<'pool, S: RestorableStream<Type = u8>> SarcIo<'pool, S> {
  pub(crate) fn names_offset(&self) -> u64 {
    names_offset(self.sfat.file_count)
  }

  pub(crate) fn data_offset(&self) -> u64 {
    self.header.data_offset as u64
  }

  /// Where the reader is, from the start of the archive.
  pub(crate) fn position(&self) -> u64 {
    self.reader.offset() - self.origin
  }

  /// Moves to `target`, from the start of the archive.
  pub(crate) async fn seek(&mut self, target: u64) -> Result<(), SarcError<'pool, S>> {
    if target < self.position() {
      match &self.entries {
        Some(entries) if target >= ENTRIES_OFFSET => self.reader.restore(entries.clone()).await?,
        _ => {
          // The snapshot was taken before the header set the byte order, and restoring it puts
          // the reader's back, so set the archive's again.
          self.reader.restore(self.start.clone()).await?;
          self.reader.set_endianness(self.header.endianness);

          // Take the snapshot of the entries again on the way past them.
          if self.entries.is_none() && target >= ENTRIES_OFFSET {
            self.reader.skip(ENTRIES_OFFSET).await?;
            self.entries = Some(self.reader.snapshot());
          }
        }
      }
    }

    let position = self.position();
    self.reader.skip(target - position).await?;
    Ok(())
  }

  pub(crate) fn hash_name(&self, name: &str) -> u32 {
    let mut hasher = SfntHasher::new(self.sfat.hash_multiplier, self.hash_mode);
    hasher.write(name.as_bytes());
    hasher.get_hash()
  }

  pub(crate) fn entry_position(index: u32) -> u64 {
    ENTRIES_OFFSET + index as u64 * SFAT_ENTRY_SIZE
  }

  /// Reads just entry `index`'s hash.
  pub(crate) async fn entry_hash(&mut self, index: u32) -> Result<u32, SarcError<'pool, S>> {
    self.seek(Self::entry_position(index)).await?;
    Ok(self.reader.get::<u32>().await?)
  }

  pub(crate) async fn entry(&mut self, index: u32) -> Result<SarcEntry, SarcError<'pool, S>> {
    let file_count = self.sfat.file_count;

    if index >= file_count as u32 {
      return Err(SarcError::NoSuchEntry { index, file_count });
    }

    let entry = self.raw_entry(index).await?;

    if entry.end < entry.start {
      return Err(SarcError::EndBeforeStart { index, start: entry.start, end: entry.end });
    }

    let end = self.data_offset() + entry.end as u64;
    if end > self.header.file_size as u64 {
      return Err(SarcError::DataOutsideArchive { index, end, file_size: self.header.file_size });
    }

    if let Some(position) = self.name_position(&entry) {
      if position >= self.data_offset() {
        return Err(SarcError::NameOutsideTable { index, position, names_end: self.data_offset() });
      }
    }

    Ok(entry)
  }

  /// Reads entry `index` without checking it: edits read entries while the archive is briefly
  /// inconsistent.
  pub(crate) async fn raw_entry(&mut self, index: u32) -> Result<SarcEntry, SarcError<'pool, S>> {
    self.seek(Self::entry_position(index)).await?;

    let SfatEntry {
      filename_hash,
      filename_attributes,
      start_offset,
      end_offset,
    } = self.reader.read().await?;

    Ok(SarcEntry {
      index,
      hash: filename_hash,
      name: filename_attributes,
      start: start_offset,
      end: end_offset,
    })
  }

  /// Where `entry`'s name is, from the start of the archive, if it has one.
  pub(crate) fn name_position(&self, entry: &SarcEntry) -> Option<u64> {
    entry.name.map(|name| self.names_offset() + name.name_offset as u64 * 4)
  }

  pub(crate) async fn name_is(&mut self, entry: &SarcEntry, name: &str) -> Result<bool, SarcError<'pool, S>> {
    let Some(position) = self.name_position(entry) else { return Ok(false) };

    self.seek(position).await?;

    let mut stored = ReadUntil::new(self.reader.stream_mut(), 0);
    let mut query = ProviderStream::new(name.as_bytes(), ReadHint::new());

    stored.eq(&mut query).await.map_err(|error| match error {
      StreamCmpError::First(error) => SarcError::Name(StreamReadError::User(error)),
      StreamCmpError::Second(_) => unreachable!("reading a name from memory can't fail"),
    })
  }

  pub(crate) async fn name_len(&mut self, entry: &SarcEntry) -> Result<Option<u64>, SarcError<'pool, S>> {
    let Some(position) = self.name_position(entry) else { return Ok(None) };

    self.seek(position).await?;

    let mut length = 0;
    while self.reader.get::<u8>().await? != 0 {
      length += 1;
    }

    Ok(Some(length))
  }

  /// Whether `entry`'s name ends with `suffix`. An entry without a name ends with nothing.
  pub(crate) async fn name_ends_with(&mut self, entry: &SarcEntry, suffix: &str) -> Result<bool, SarcError<'pool, S>> {
    let (Some(position), Some(length)) = (self.name_position(entry), self.name_len(entry).await?) else { return Ok(false) };

    let Some(skip) = length.checked_sub(suffix.len() as u64) else { return Ok(false) };

    self.seek(position + skip).await?;
    for &expected in suffix.as_bytes() {
      if self.reader.get::<u8>().await? != expected {
        return Ok(false);
      }
    }

    Ok(true)
  }

  /// The index of the first entry whose hash isn't below `hash`.
  pub(crate) async fn lower_bound(&mut self, hash: u32) -> Result<u32, SarcError<'pool, S>> {
    let (mut low, mut high) = (0, self.sfat.file_count as u32);

    while low < high {
      let middle = (low + high) / 2;
      if self.entry_hash(middle).await? < hash {
        low = middle + 1;
      } else {
        high = middle;
      }
    }

    Ok(low)
  }

  /// The index of the first entry whose hash is above `hash`.
  pub(crate) async fn upper_bound(&mut self, hash: u32) -> Result<u32, SarcError<'pool, S>> {
    let (mut low, mut high) = (0, self.sfat.file_count as u32);

    while low < high {
      let middle = (low + high) / 2;
      if self.entry_hash(middle).await? <= hash {
        low = middle + 1;
      } else {
        high = middle;
      }
    }

    Ok(low)
  }

  /// Finds `name` the way sead does: a binary search for its hash, then, if entries share the
  /// hash, a walk through them comparing names.
  pub(crate) async fn find(&mut self, name: &str) -> Result<Option<SarcEntry>, SarcError<'pool, S>> {
    let hash = self.hash_name(name);
    let file_count = self.sfat.file_count as u32;

    let (mut low, mut high) = (0, file_count);
    let found = loop {
      if low >= high {
        return Ok(None);
      }

      let middle = (low + high) / 2;
      let middle_hash = self.entry_hash(middle).await?;

      if middle_hash < hash {
        low = middle + 1;
      } else if middle_hash > hash {
        high = middle;
      } else {
        break middle;
      }
    };

    let entry = self.entry(found).await?;

    // Without a name, the hash is all there is to go on.
    let Some(attributes) = entry.name else { return Ok(Some(entry)) };

    // Collisions are numbered from 1, so this is the first entry with the hash.
    let sequence = attributes.sequence.get();
    let mut index = found.checked_sub(sequence as u32 - 1).ok_or(SarcError::CollisionIndexTooLarge { index: found, sequence })?;

    while index < file_count {
      let entry = self.entry(index).await?;

      if entry.hash != hash {
        return Ok(None);
      }

      if self.name_is(&entry, name).await? {
        return Ok(Some(entry));
      }

      index += 1;
    }

    Ok(None)
  }
}
