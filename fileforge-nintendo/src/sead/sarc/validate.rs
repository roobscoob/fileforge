use core::hash::Hasher;

use fileforge::{binary_reader::PrimitiveReader, stream::RestorableStream};

use crate::sead::sarc::{sfat::name_table::hasher::SfntHasher, AlignmentPolicy, Sarc, SarcError};

impl<'pool, S: RestorableStream<Type = u8>, A: AlignmentPolicy> Sarc<'pool, S, A> {
  /// Checks the whole archive: every entry is inside it, the entries are sorted by hash, entries
  /// that share a hash are numbered and named properly, and each name matches its hash.
  ///
  /// Reading a file only checks its own entry; this checks what sead relies on to find them.
  pub async fn validate(&mut self) -> Result<(), SarcError<'pool, S>> {
    let io = &mut self.io;
    let file_count = io.sfat.file_count as u32;

    let mut previous_hash = None;
    // How many entries so far share the current hash, and whether any of them has no name.
    let mut run = 0;
    let mut run_unnamed = false;

    for index in 0..file_count {
      let entry = io.entry(index).await?;

      if previous_hash.is_some_and(|previous| entry.hash < previous) {
        return Err(SarcError::NotSorted { index });
      }

      if previous_hash == Some(entry.hash) {
        run += 1;
      } else {
        run = 1;
        run_unnamed = false;
      }
      previous_hash = Some(entry.hash);

      match entry.name {
        None => run_unnamed = true,
        Some(name) if name.sequence.get() as u32 != run => {
          return Err(SarcError::WrongCollisionIndex {
            index,
            expected: run,
            found: name.sequence.get(),
          });
        }
        Some(_) => {}
      }

      if run > 1 && run_unnamed {
        return Err(SarcError::UnnamedCollision { index });
      }

      if let Some(position) = io.name_position(&entry) {
        io.seek(position).await?;

        let mut hasher = SfntHasher::new(io.sfat.hash_multiplier, io.hash_mode);
        loop {
          let byte = io.reader.get::<u8>().await?;
          if byte == 0 {
            break;
          }
          hasher.write(&[byte]);
        }

        let computed = hasher.get_hash();
        if computed != entry.hash {
          return Err(SarcError::NameHashMismatch { index, stored: entry.hash, computed });
        }
      }
    }

    Ok(())
  }
}
