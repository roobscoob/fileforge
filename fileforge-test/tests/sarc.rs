//! Reading SARC archives. The archives in `binaries/sarc` are written by oead (see `generate.py`),
//! an independent implementation; `SkyWorldHomeStageMap.szs` is a real one from Super Mario Odyssey.

use fileforge::{
  binary_reader::{endianness::Endianness, BinaryReader},
  provider::hint::ReadHint,
  stream::builtin::provider::ProviderStream,
};
use fileforge_nintendo::sead::{
  sarc::{
    header::readable::SarcHeaderReadError,
    readable::SarcReadError,
    sfat::{header::readable::SfatHeaderReadError, name_table::header::SfntHeaderReadError},
    Sarc,
  },
  yaz0::{readable::Immutable, Yaz0Stream},
};
use fileforge_std::{byte_order_mark::error::ByteOrderMarkError, magic::MagicError};

const THREE_FILES_LE: &[u8] = include_bytes!("../binaries/sarc/three-files-le.sarc");
const THREE_FILES_BE: &[u8] = include_bytes!("../binaries/sarc/three-files-be.sarc");
const EMPTY_LE: &[u8] = include_bytes!("../binaries/sarc/empty-le.sarc");
const SKY_WORLD: &[u8] = include_bytes!("../binaries/SkyWorldHomeStageMap.szs");

fn runtime() -> tokio::runtime::Runtime {
  tokio::runtime::Builder::new_current_thread().build().unwrap()
}

/// Opens `bytes` as a SARC archive, and passes it (or the error) to `check`.
fn open<R>(bytes: &[u8], check: impl FnOnce(Result<&Sarc<'_, ProviderStream<&[u8]>>, &SarcReadError<'_, ProviderStream<&[u8]>>>) -> R) -> R {
  runtime().block_on(async {
    let reader = BinaryReader::new_from_provider(bytes, Endianness::BigEndian, ReadHint::new());
    let sarc = reader.into::<Sarc<_>>().await;
    check(sarc.as_ref())
  })
}

#[test]
fn opens_archives_written_by_oead() {
  for (name, bytes, endianness) in [("little-endian", THREE_FILES_LE, Endianness::LittleEndian), ("big-endian", THREE_FILES_BE, Endianness::BigEndian)] {
    open(bytes, |sarc| {
      let sarc = sarc.unwrap_or_else(|_| panic!("the {name} archive should open"));
      assert_eq!(sarc.header().endianness, endianness, "{name}");
      assert_eq!(sarc.file_count(), 3, "{name}");
      assert_eq!(sarc.hash_multiplier(), 0x65, "{name}");
      assert_eq!(sarc.header().file_size, 212, "{name}");
      assert_eq!(sarc.entries_offset(), 0x20, "{name}");
      assert_eq!(sarc.names_offset(), 0x58, "{name}");
      assert_eq!(sarc.data_offset(), 0x8C, "{name}");
    });
  }
}

#[test]
fn opens_an_empty_archive() {
  open(EMPTY_LE, |sarc| {
    let sarc = sarc.unwrap_or_else(|_| panic!("the empty archive should open"));
    assert_eq!(sarc.file_count(), 0);
    assert_eq!(sarc.names_offset(), 0x28);
    assert_eq!(sarc.data_offset(), 0x28);
  });
}

#[test]
fn opens_a_real_archive_through_yaz0() {
  runtime().block_on(async {
    let reader = BinaryReader::new_from_provider(SKY_WORLD, Endianness::BigEndian, ReadHint::new());
    let yaz0 = reader.into_with::<Yaz0Stream<_, Immutable>>(Immutable).await.unwrap_or_else(|_| panic!("the Yaz0 stream should open"));

    let sarc = BinaryReader::new(yaz0, Endianness::BigEndian).into::<Sarc<_>>().await.unwrap_or_else(|_| panic!("the archive should open"));

    // As oead reads it.
    assert_eq!(sarc.header().endianness, Endianness::LittleEndian);
    assert_eq!(sarc.file_count(), 8);
    assert_eq!(sarc.data_offset(), 0x200);
    assert_eq!(sarc.header().file_size, 3_128_776);
  });
}

/// `THREE_FILES_LE` with `patch` written at `offset`.
fn patched(offset: usize, patch: &[u8]) -> Vec<u8> {
  let mut bytes = THREE_FILES_LE.to_vec();
  bytes[offset..offset + patch.len()].copy_from_slice(patch);
  bytes
}

#[test]
fn rejects_a_wrong_magic() {
  open(&patched(0x0, b"BAD!"), |sarc| assert!(matches!(sarc, Err(SarcReadError::Header(SarcHeaderReadError::Magic(MagicError::Invalid { .. }))))));
}

#[test]
fn rejects_a_wrong_header_length() {
  open(&patched(0x4, &0x10u16.to_le_bytes()), |sarc| {
    assert!(matches!(sarc, Err(SarcReadError::Header(SarcHeaderReadError::WrongHeaderLength(length))) if **length == 0x10))
  });
}

#[test]
fn rejects_a_wrong_byte_order_mark() {
  open(&patched(0x6, &[0x12, 0x34]), |sarc| assert!(matches!(sarc, Err(SarcReadError::Header(SarcHeaderReadError::BOM(ByteOrderMarkError::Invalid(_)))))));
}

#[test]
fn rejects_an_unsupported_version() {
  open(&patched(0x10, &0x200u16.to_le_bytes()), |sarc| {
    assert!(matches!(sarc, Err(SarcReadError::Header(SarcHeaderReadError::UnsupportedVersion(version))) if **version == 0x200))
  });
}

#[test]
fn rejects_a_wrong_sfat_header_length() {
  open(&patched(0x18, &0x10u16.to_le_bytes()), |sarc| {
    assert!(matches!(sarc, Err(SarcReadError::SfatHeader(SfatHeaderReadError::WrongHeaderLength(length))) if **length == 0x10))
  });
}

#[test]
fn rejects_too_many_files() {
  open(&patched(0x1A, &0x4000u16.to_le_bytes()), |sarc| {
    assert!(matches!(sarc, Err(SarcReadError::SfatHeader(SfatHeaderReadError::TooManyFiles(count))) if **count == 0x4000))
  });
}

#[test]
fn rejects_a_missing_sfnt() {
  open(&patched(0x50, b"BAD!"), |sarc| assert!(matches!(sarc, Err(SarcReadError::SfntHeader(SfntHeaderReadError::Magic(MagicError::Invalid { .. }))))));
}

#[test]
fn rejects_a_wrong_sfnt_header_length() {
  open(&patched(0x54, &0x10u16.to_le_bytes()), |sarc| {
    assert!(matches!(sarc, Err(SarcReadError::SfntHeader(SfntHeaderReadError::WrongHeaderLength(length))) if **length == 0x10))
  });
}

#[test]
fn rejects_data_that_starts_inside_the_tables() {
  open(&patched(0xC, &0x10u32.to_le_bytes()), |sarc| {
    assert!(matches!(sarc, Err(SarcReadError::DataInsideTables { data_offset, tables_end: 0x58 }) if **data_offset == 0x10))
  });

  // Right where the tables end is fine: the archive just has no names.
  open(&patched(0xC, &0x58u32.to_le_bytes()), |sarc| assert!(sarc.is_ok()));
}

#[test]
fn every_truncation_of_the_tables_is_an_error() {
  for length in 0..0x58 {
    open(&THREE_FILES_LE[..length], |sarc| assert!(sarc.is_err(), "an archive cut to {length} bytes should not open"));
  }
}
