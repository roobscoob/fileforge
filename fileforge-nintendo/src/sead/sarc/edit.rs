//! Changing a SARC archive: resizing files, adding and removing them, creating an archive, and
//! realigning one to a new policy.
//!
//! An edit is several writes, and between them the archive can briefly be inconsistent: the
//! tables and the data are updated one after the other.

use fileforge::{
  binary_reader::{
    endianness::Endianness,
    error::{common::Write, primitive_name_annotation::PrimitiveName, SetPrimitiveError},
    readable::IntoReadable,
    BinaryReader, PrimitiveWriter,
  },
  diagnostic::pool::DiagnosticPoolProvider,
  error::{
    ext::annotations::annotated::Annotated,
    render::{
      buffer::cell::tag::builtin::report::{REPORT_ERROR_TEXT, REPORT_INFO_LINE_TEXT},
      builtin::{number::formatted_unsigned::FormattedUnsigned, text::r#const::ConstText},
    },
    report::Report,
    FileforgeError,
  },
  stream::{
    error::{stream_overwrite::StreamOverwriteError, user_overwrite::UserOverwriteError},
    MutableStream, ResizableStream, RestorableStream,
  },
};
use fileforge_macros::{story, text};

use crate::{
  report::render_with_context,
  sead::sarc::{
    align::{AlignmentQuery, FileInfo, QueryName},
    header::{SARC_HEADER_SIZE, SARC_VERSION},
    readable::SarcReadError,
    sfat::{
      entry::{attributes::FilenameAttributes, SFAT_ENTRY_SIZE},
      header::{SFAT_HEADER_SIZE, SFAT_MAX_FILES},
      name_table::header::SFNT_HEADER_SIZE,
    },
    AlignmentPolicy, Sarc, SarcEntry, SarcError, SarcIo,
  },
};

/// Where the file size sits in the SARC header.
const FILE_SIZE_POSITION: u64 = 0x8;
/// Where the data offset sits in the SARC header.
const DATA_OFFSET_POSITION: u64 = 0xC;
/// Where the file count sits in the SFAT header.
const FILE_COUNT_POSITION: u64 = SARC_HEADER_SIZE as u64 + 0x6;

/// A failure changing a SARC archive.
#[story("bad alignment", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::BadAlignment { alignment: 12 };
  error
})]
#[story("invalid name", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::InvalidName;
  error
})]
#[story("name already exists", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::NameExists { index: 2 };
  error
})]
#[story("hash shared with an unnamed file", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::CollidesWithUnnamed { index: 1 };
  error
})]
#[story("too many files", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::TooManyFiles;
  error
})]
#[story("too many files share a hash", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::TooManyCollisions;
  error
})]
#[story("archive too large", {
  let error: SarcEditError<'_, StoryStream> = SarcEditError::TooLarge;
  error
})]
pub enum SarcEditError<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> {
  /// Reading the archive failed.
  Access(SarcError<'pool, S>),
  /// Opening the archive after creating it failed.
  Open(SarcReadError<'pool, S>),
  /// Writing a value, such as an entry's offset, failed.
  Write(Annotated<PrimitiveName<Write>, SetPrimitiveError<'pool, S::MutateError>>),
  /// Inserting or removing bytes failed.
  Resize(StreamOverwriteError<S::OverwriteError>),

  /// The alignment policy gave an alignment that isn't a power of two.
  BadAlignment { alignment: u32 },
  /// A name was empty, or contained a null byte.
  InvalidName,
  /// A file with the name already exists, as entry `index`.
  NameExists { index: u32 },
  /// The name's hash is shared with entry `index`, which has no name.
  CollidesWithUnnamed { index: u32 },
  /// The archive already has as many files as sead reads.
  TooManyFiles,
  /// 255 files already share the name's hash.
  TooManyCollisions,
  /// An offset in the archive wouldn't fit in its field.
  TooLarge,
}

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> UserOverwriteError for SarcEditError<'pool, S> {}

const WRITING: ConstText = ConstText::new("This happened while writing to a SARC archive.", &REPORT_INFO_LINE_TEXT);
const RESIZING: ConstText = ConstText::new("This happened while making room in a SARC archive, or removing some.", &REPORT_INFO_LINE_TEXT);
const OPENING: ConstText = ConstText::new("This happened while opening a SARC archive that was just created.", &REPORT_INFO_LINE_TEXT);
const INVALID_NAME: ConstText = ConstText::new("A file's name can't be empty, or contain a null byte.", &REPORT_ERROR_TEXT);
const UNNAMED: ConstText = ConstText::new("An entry without a name can only be found when no other entry shares its hash.", &REPORT_INFO_LINE_TEXT);
const TOO_MANY_FILES: ConstText = ConstText::new("The archive already has 16,383 files, the most sead reads.", &REPORT_ERROR_TEXT);
const TOO_MANY_COLLISIONS: ConstText = ConstText::new("255 files already share this name's hash, the most a SARC archive can number.", &REPORT_ERROR_TEXT);
const TOO_LARGE: ConstText = ConstText::new("After this edit, an offset in the archive wouldn't fit in its field.", &REPORT_ERROR_TEXT);

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> FileforgeError for SarcEditError<'pool, S> {
  fn render_into_report<P: DiagnosticPoolProvider + Clone, const ITEM_NAME_SIZE: usize>(&self, provider: P, callback: impl for<'tag, 'b> FnOnce(Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()) {
    match self {
      Self::Access(error) => error.render_into_report(provider, callback),
      Self::Open(error) => render_with_context(error, &OPENING, provider, callback),
      Self::Write(error) => render_with_context(error, &WRITING, provider, callback),
      Self::Resize(error) => render_with_context(error, &RESIZING, provider, callback),
      Self::BadAlignment { alignment } => {
        let alignment = FormattedUnsigned::new(*alignment as u128);
        let text = text!([&REPORT_ERROR_TEXT] "The archive's alignment policy asked for an alignment of {&alignment}, which isn't a power of two.");

        Report::new::<Self>(provider, &"Invalid alignment").with_info_line(&text).apply(callback)
      }
      Self::InvalidName => Report::new::<Self>(provider, &"Invalid file name").with_info_line(&INVALID_NAME).apply(callback),
      Self::NameExists { index } => {
        let index = FormattedUnsigned::new(*index as u128);
        let text = text!([&REPORT_ERROR_TEXT] "The archive already has a file with this name, as entry {&index}.");

        Report::new::<Self>(provider, &"File already exists").with_info_line(&text).apply(callback)
      }
      Self::CollidesWithUnnamed { index } => {
        let index = FormattedUnsigned::new(*index as u128);
        let text = text!([&REPORT_ERROR_TEXT] "The name's hash is shared with entry {&index}, which has no name.");

        Report::new::<Self>(provider, &"Name collides with an unnamed file").with_info_line(&text).with_info_line(&UNNAMED).apply(callback)
      }
      Self::TooManyFiles => Report::new::<Self>(provider, &"Too many files").with_info_line(&TOO_MANY_FILES).apply(callback),
      Self::TooManyCollisions => Report::new::<Self>(provider, &"Too many files share a hash").with_info_line(&TOO_MANY_COLLISIONS).apply(callback),
      Self::TooLarge => Report::new::<Self>(provider, &"SARC archive too large").with_info_line(&TOO_LARGE).apply(callback),
    }
  }
}

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> From<SarcError<'pool, S>> for SarcEditError<'pool, S> {
  fn from(value: SarcError<'pool, S>) -> Self {
    Self::Access(value)
  }
}

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> From<Annotated<PrimitiveName<Write>, SetPrimitiveError<'pool, S::MutateError>>> for SarcEditError<'pool, S> {
  fn from(value: Annotated<PrimitiveName<Write>, SetPrimitiveError<'pool, S::MutateError>>) -> Self {
    Self::Write(value)
  }
}

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> From<StreamOverwriteError<S::OverwriteError>> for SarcEditError<'pool, S> {
  fn from(value: StreamOverwriteError<S::OverwriteError>) -> Self {
    Self::Resize(value)
  }
}

fn round_up(value: u64, step: u64) -> u64 {
  value.div_ceil(step) * step
}

fn round_down(value: u64, step: u64) -> u64 {
  value / step * step
}

fn to_u32<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream>(value: u64) -> Result<u32, SarcEditError<'pool, S>> {
  value.try_into().map_err(|_| SarcEditError::TooLarge)
}

/// The bytes of an archive with no files.
fn empty_archive(endianness: Endianness) -> [u8; 0x28] {
  let u16 = |value: u16| match endianness {
    Endianness::BigEndian => value.to_be_bytes(),
    Endianness::LittleEndian => value.to_le_bytes(),
  };
  let u32 = |value: u32| match endianness {
    Endianness::BigEndian => value.to_be_bytes(),
    Endianness::LittleEndian => value.to_le_bytes(),
  };

  let size = 0x28u32;
  let mut bytes = [0u8; 0x28];
  let mut at = 0;
  let mut put = |part: &[u8]| {
    bytes[at..at + part.len()].copy_from_slice(part);
    at += part.len();
  };

  put(b"SARC");
  put(&u16(SARC_HEADER_SIZE));
  put(&u16(0xFEFF));
  put(&u32(size));
  put(&u32(size));
  put(&u16(SARC_VERSION));
  put(&u16(0));

  put(b"SFAT");
  put(&u16(SFAT_HEADER_SIZE));
  put(&u16(0));
  put(&u32(0x65));

  put(b"SFNT");
  put(&u16(SFNT_HEADER_SIZE));
  put(&u16(0));

  bytes
}

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream> SarcIo<'pool, S> {
  async fn write_u32(&mut self, position: u64, value: u32) -> Result<(), SarcEditError<'pool, S>> {
    self.seek(position).await?;
    self.entries = None;
    Ok(self.reader.set(value).await?)
  }

  async fn write_u16(&mut self, position: u64, value: u16) -> Result<(), SarcEditError<'pool, S>> {
    self.seek(position).await?;
    self.entries = None;
    Ok(self.reader.set(value).await?)
  }

  async fn write_entry(&mut self, entry: &SarcEntry) -> Result<(), SarcEditError<'pool, S>> {
    let position = Self::entry_position(entry.index);

    self.write_u32(position, entry.hash).await?;
    self.write_u32(position + 4, FilenameAttributes::to_bits(entry.name)).await?;
    self.write_u32(position + 8, entry.start).await?;
    self.write_u32(position + 12, entry.end).await
  }

  async fn write_file_size(&mut self, file_size: u32) -> Result<(), SarcEditError<'pool, S>> {
    self.write_u32(FILE_SIZE_POSITION, file_size).await?;
    self.header.file_size = file_size;
    Ok(())
  }

  async fn insert_zeros(&mut self, position: u64, mut count: u64) -> Result<(), SarcEditError<'pool, S>> {
    self.seek(position).await?;
    self.entries = None;

    while count >= 64 {
      self.reader.stream_mut().overwrite(0, [0u8; 64]).await?;
      count -= 64;
    }

    while count > 0 {
      self.reader.stream_mut().overwrite(0, [0u8; 1]).await?;
      count -= 1;
    }

    Ok(())
  }

  async fn insert_bytes(&mut self, position: u64, bytes: &[u8]) -> Result<(), SarcEditError<'pool, S>> {
    self.seek(position).await?;
    self.entries = None;

    for chunk in bytes.chunks(16) {
      match <[u8; 16]>::try_from(chunk) {
        Ok(chunk) => self.reader.stream_mut().overwrite(0, chunk).await?,
        Err(_) => {
          for &byte in chunk {
            self.reader.stream_mut().overwrite(0, [byte]).await?;
          }
        }
      }
    }

    Ok(())
  }

  async fn remove(&mut self, position: u64, count: u64) -> Result<(), SarcEditError<'pool, S>> {
    self.seek(position).await?;
    self.entries = None;
    Ok(self.reader.stream_mut().overwrite(count, []).await?)
  }

  /// Where the name table's names end: after the last name, padded to 4 bytes.
  async fn names_end(&mut self) -> Result<u64, SarcEditError<'pool, S>> {
    let mut end = self.names_offset();

    for index in 0..self.sfat.file_count as u32 {
      let entry = self.raw_entry(index).await?;

      if let (Some(position), Some(length)) = (self.name_position(&entry), self.name_len(&entry).await?) {
        end = end.max(position + round_up(length + 1, 4));
      }
    }

    Ok(end)
  }
}

impl<'pool, S: RestorableStream<Type = u8> + MutableStream + ResizableStream, A: AlignmentPolicy> Sarc<'pool, S, A> {
  /// Creates an empty archive at `reader`'s position, which should be the end of its stream,
  /// and opens it.
  pub async fn create(mut reader: BinaryReader<'pool, S>, endianness: Endianness, policy: A) -> Result<Self, SarcEditError<'pool, S>> {
    let start = reader.snapshot();

    reader.stream_mut().overwrite(0, empty_archive(endianness)).await?;
    reader.restore(start).await.map_err(|error| SarcEditError::Access(SarcError::Restore(error)))?;

    <Self as IntoReadable<'pool, S>>::read(reader, policy).await.map_err(SarcEditError::Open)
  }

  /// Asks the policy what `info`'s file needs, and checks the answer.
  async fn alignment_of(&mut self, info: FileInfo, name: Option<&str>) -> Result<u64, SarcEditError<'pool, S>> {
    let Sarc { io, policy } = self;

    let mut query = AlignmentQuery {
      info,
      name: match name {
        Some(name) => QueryName::Given(name),
        None => QueryName::Stored(io),
      },
    };

    let alignment = policy.alignment(&mut query).await?;

    if !alignment.is_power_of_two() {
      return Err(SarcEditError::BadAlignment { alignment });
    }

    Ok(alignment as u64)
  }

  async fn alignment_of_entry(&mut self, entry: SarcEntry) -> Result<u64, SarcEditError<'pool, S>> {
    let position = self.io.data_offset() + entry.start as u64;

    self
      .alignment_of(
        FileInfo {
          entry: Some(entry),
          hash: entry.hash,
          position: Some(position),
        },
        None,
      )
      .await
  }

  /// The largest alignment of the files, other than `except`, that start at or after `from` in
  /// the data section, and where the first of them starts.
  async fn files_after(&mut self, except: Option<u32>, from: u64) -> Result<(Option<u64>, u64), SarcEditError<'pool, S>> {
    let mut first = None;
    let mut step = 1;

    for index in 0..self.io.sfat.file_count as u32 {
      let entry = self.io.raw_entry(index).await?;

      if Some(index) == except || (entry.start as u64) < from {
        continue;
      }

      first = Some(first.map_or(entry.start as u64, |first: u64| first.min(entry.start as u64)));
      step = step.max(self.alignment_of_entry(entry).await?);
    }

    Ok((first, step))
  }

  /// Replaces `length` bytes at `at` (inside `entry`'s data) with `data`, and moves the files
  /// after it by a multiple of their alignment. Returns the entry as it is afterwards.
  pub(crate) async fn resize_within<const SIZE: usize>(&mut self, entry: SarcEntry, at: u64, length: u64, data: [u8; SIZE]) -> Result<SarcEntry, SarcEditError<'pool, S>> {
    self.io.seek(at).await?;
    self.io.entries = None;
    self.io.reader.stream_mut().overwrite(length, data).await?;

    let delta = SIZE as i64 - length as i64;

    if delta == 0 {
      return Ok(entry);
    }

    let old_end = entry.end as u64;
    let new_end = (old_end as i64 + delta) as u64;
    let (next, step) = self.files_after(Some(entry.index), old_end).await?;

    // How far the files after this one move. Growing uses up the padding after the file first.
    let shift = match next {
      None => delta,
      Some(next) if delta > 0 => {
        let gap = next - old_end;
        (delta as u64).checked_sub(gap).map_or(0, |needed| round_up(needed, step) as i64)
      }
      Some(_) => -(round_down(delta.unsigned_abs(), step) as i64),
    };

    // The padding after the file makes up the difference.
    let padding = self.io.data_offset() + new_end;
    match shift - delta {
      adjust if adjust > 0 => self.io.insert_zeros(padding, adjust as u64).await?,
      adjust if adjust < 0 => self.io.remove(padding, adjust.unsigned_abs()).await?,
      _ => {}
    }

    let entry = SarcEntry { end: to_u32(new_end)?, ..entry };
    self.io.write_entry(&entry).await?;

    if next.is_some() && shift != 0 {
      for index in 0..self.io.sfat.file_count as u32 {
        let other = self.io.raw_entry(index).await?;

        if index != entry.index && other.start as u64 >= old_end {
          let other = SarcEntry {
            start: to_u32((other.start as i64 + shift) as u64)?,
            end: to_u32((other.end as i64 + shift) as u64)?,
            ..other
          };
          self.io.write_entry(&other).await?;
        }
      }
    }

    let file_size = to_u32((self.io.header.file_size as i64 + shift) as u64)?;
    self.io.write_file_size(file_size).await?;

    Ok(entry)
  }

  /// Adds an empty file called `name` at the end of the archive, and returns its entry. Give it
  /// data by opening it with [`file`](Sarc::file) and writing to it.
  ///
  /// Adding a file moves the entries after it along by one, so entries read before are stale.
  pub async fn add_file(&mut self, name: &str) -> Result<SarcEntry, SarcEditError<'pool, S>> {
    if name.is_empty() || name.contains('\0') {
      return Err(SarcEditError::InvalidName);
    }

    if self.io.sfat.file_count >= SFAT_MAX_FILES {
      return Err(SarcEditError::TooManyFiles);
    }

    if let Some(existing) = self.io.find(name).await? {
      return Err(SarcEditError::NameExists { index: existing.index });
    }

    let hash = self.io.hash_name(name);
    let lower = self.io.lower_bound(hash).await?;
    let index = self.io.upper_bound(hash).await?;

    for other in lower..index {
      if self.io.raw_entry(other).await?.name.is_none() {
        return Err(SarcEditError::CollidesWithUnnamed { index: other });
      }
    }

    let sequence = u8::try_from(index - lower + 1).ok().and_then(core::num::NonZero::new).ok_or(SarcEditError::TooManyCollisions)?;

    let names_offset = self.io.names_offset();
    let names_end = self.io.names_end().await?;
    let name_offset = ((names_end - names_offset) / 4) as u32;

    if name_offset > 0xFF_FFFF {
      return Err(SarcEditError::TooLarge);
    }

    // The tables grow by an entry and a name. They use up the padding before the data first;
    // past that, the data moves by a multiple of the largest alignment any file needs.
    let name_size = round_up(name.len() as u64 + 1, 4);
    let needed = SFAT_ENTRY_SIZE + name_size;
    let data_offset = self.io.data_offset();
    let slack = data_offset - names_end;

    let shift = match needed.checked_sub(slack) {
      None | Some(0) => 0,
      Some(short) => {
        let (_, step) = self.files_after(None, 0).await?;
        round_up(short, step)
      }
    };

    // The new file goes at the end, where its alignment says.
    let new_data_offset = data_offset + shift;
    let data_end = self.io.header.file_size as u64 - data_offset;
    let alignment = self.alignment_of(FileInfo { entry: None, hash, position: None }, Some(name)).await?;
    let start = round_up(new_data_offset + data_end, alignment) - new_data_offset;

    let entry = SarcEntry {
      index,
      hash,
      name: Some(FilenameAttributes { sequence, name_offset }),
      start: to_u32(start)?,
      end: to_u32(start)?,
    };
    let file_size = to_u32(new_data_offset + start)?;
    let data_offset_field = to_u32(new_data_offset)?;

    // The entry, then the name (which the entry has moved along), then the padding before the
    // data, then the padding before the new file.
    self.io.insert_zeros(SarcIo::<S>::entry_position(index), SFAT_ENTRY_SIZE).await?;
    self.io.write_entry(&entry).await?;

    let name_position = names_end + SFAT_ENTRY_SIZE;
    self.io.insert_bytes(name_position, name.as_bytes()).await?;
    self.io.insert_zeros(name_position + name.len() as u64, name_size - name.len() as u64).await?;

    let padding = names_end + needed;
    match shift as i64 - needed as i64 {
      adjust if adjust > 0 => self.io.insert_zeros(padding, adjust as u64).await?,
      adjust if adjust < 0 => self.io.remove(padding, adjust.unsigned_abs()).await?,
      _ => {}
    }

    self.io.insert_zeros(new_data_offset + data_end, start - data_end).await?;

    self.io.sfat.file_count += 1;
    self.io.write_u16(FILE_COUNT_POSITION, self.io.sfat.file_count).await?;
    self.io.header.data_offset = data_offset_field;
    self.io.write_u32(DATA_OFFSET_POSITION, data_offset_field).await?;
    self.io.write_file_size(file_size).await?;

    Ok(entry)
  }

  /// Removes entry `index` and its data. Its name stays in the name table, unused.
  ///
  /// Removing a file moves the entries after it back by one, so entries read before are stale.
  pub async fn remove_file(&mut self, index: u32) -> Result<(), SarcEditError<'pool, S>> {
    let entry = self.io.entry(index).await?;

    if entry.len() > 0 {
      let at = self.io.data_offset() + entry.start as u64;
      self.resize_within(entry, at, entry.len(), []).await?;
    }

    // The entries after it with the same hash are numbered one lower.
    for other in index + 1..self.io.sfat.file_count as u32 {
      let other = self.io.raw_entry(other).await?;

      if other.hash != entry.hash {
        break;
      }

      if let Some(name) = other.name {
        let sequence = core::num::NonZero::new(name.sequence.get() - 1).unwrap_or(name.sequence);
        self.io.write_entry(&SarcEntry { name: Some(FilenameAttributes { sequence, ..name }), ..other }).await?;
      }
    }

    // Take the entry out, and put the same space back as padding before the data, so the data
    // doesn't move.
    self.io.remove(SarcIo::<S>::entry_position(index), SFAT_ENTRY_SIZE).await?;
    self.io.insert_zeros(self.io.data_offset() - SFAT_ENTRY_SIZE, SFAT_ENTRY_SIZE).await?;

    self.io.sfat.file_count -= 1;
    self.io.write_u16(FILE_COUNT_POSITION, self.io.sfat.file_count).await?;

    Ok(())
  }

  /// Replaces the archive's alignment policy, without moving anything. Follow it with
  /// [`realign`](Sarc::realign) to bring the files in line with it.
  pub fn with_policy<B: AlignmentPolicy>(self, policy: B) -> Sarc<'pool, S, B> {
    Sarc { io: self.io, policy }
  }

  /// Moves each file that doesn't have the alignment the policy gives it forward, just far enough
  /// that it does. Padding is only ever added, never removed.
  pub async fn realign(&mut self) -> Result<(), SarcEditError<'pool, S>> {
    let mut previous: Option<(u32, u32)> = None;

    loop {
      // The next file in data order, by where it starts and then by index.
      let mut next: Option<SarcEntry> = None;

      for index in 0..self.io.sfat.file_count as u32 {
        let entry = self.io.raw_entry(index).await?;
        let key = (entry.start, index);

        if previous.is_none_or(|previous| key > previous) && next.is_none_or(|next| key < (next.start, next.index)) {
          next = Some(entry);
        }
      }

      let Some(entry) = next else { return Ok(()) };

      let position = self.io.data_offset() + entry.start as u64;
      let alignment = self.alignment_of_entry(entry).await?;
      let padding = round_up(position, alignment) - position;

      if padding > 0 {
        self.io.insert_zeros(position, padding).await?;

        for index in 0..self.io.sfat.file_count as u32 {
          let other = self.io.raw_entry(index).await?;

          if other.start >= entry.start {
            let other = SarcEntry {
              start: to_u32(other.start as u64 + padding)?,
              end: to_u32(other.end as u64 + padding)?,
              ..other
            };
            self.io.write_entry(&other).await?;
          }
        }

        let file_size = to_u32(self.io.header.file_size as u64 + padding)?;
        self.io.write_file_size(file_size).await?;
      }

      previous = Some((to_u32(entry.start as u64 + padding)?, entry.index));
    }
  }
}
