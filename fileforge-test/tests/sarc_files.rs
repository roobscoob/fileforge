//! Reading and editing the files in SARC archives.
//!
//! The archives in `binaries/sarc` are written by oead (see `generate.py`); `SkyWorldHomeStageMap.szs`
//! is a real one from Super Mario Odyssey. Set `OEAD_PYTHON` to a Python with oead installed to also
//! have oead read back the archives these tests edit.

use std::{collections::BTreeMap, io::Cursor};

use fileforge::{
  binary_reader::{endianness::Endianness, BinaryReader},
  diagnostic::pool::dynamic::DynamicDiagnosticPool,
  error::{render::buffer::cell::tag::context::RenderMode, FileforgeError, RenderableError},
  provider::hint::ReadHint,
  stream::{builtin::provider::ProviderStream, extensions::readable::ReadableStreamExt, MutableStream, ReadableStream, ResizableStream, RestorableStream, SINGLE},
};
use fileforge_nintendo::sead::{
  sarc::{edit::SarcEditError, AlignmentPolicy, Fixed, Preserve, Sarc, SarcEntry, SarcError},
  yaz0::{
    readable::{Immutable, Mutable},
    Yaz0Stream,
  },
};

const THREE_FILES_LE: &[u8] = include_bytes!("../binaries/sarc/three-files-le.sarc");
const THREE_FILES_BE: &[u8] = include_bytes!("../binaries/sarc/three-files-be.sarc");
const COLLISIONS: &[u8] = include_bytes!("../binaries/sarc/collisions-le.sarc");
const COLLISIONS_OEAD: &[u8] = include_bytes!("../binaries/sarc/collisions-oead-le.sarc");
const UNICODE: &[u8] = include_bytes!("../binaries/sarc/unicode-le.sarc");
const SKY_WORLD: &[u8] = include_bytes!("../binaries/SkyWorldHomeStageMap.szs");

/// The files `generate.py` puts in every archive.
fn common_files() -> BTreeMap<String, Vec<u8>> {
  BTreeMap::from([
    ("Model/Actor.bfres".to_string(), (0..40).collect()),
    ("Param/Actor.byml".to_string(), [&b"BY"[..], &[0; 18]].concat()),
    ("Readme.txt".to_string(), b"hello, sarc\n".to_vec()),
  ])
}

fn collision_files() -> BTreeMap<String, Vec<u8>> {
  let mut files = common_files();
  files.insert("c/aycpvjtnn".into(), b"first of the first pair".to_vec());
  files.insert("c/kevbqeqbp".into(), b"second of the first pair".to_vec());
  files.insert("c/gwbuwf".into(), b"first of the second pair".to_vec());
  files.insert("c/lskfark".into(), b"second of the second pair".to_vec());
  files
}

fn runtime() -> tokio::runtime::Runtime {
  tokio::runtime::Builder::new_current_thread().build().unwrap()
}

/// `result`'s value, or a panic showing the error's report.
fn expect<T, E: FileforgeError>(result: Result<T, E>) -> T {
  result.unwrap_or_else(|error| {
    let pool = DynamicDiagnosticPool::new();
    panic!("{:?}", RenderableError::<64, _, _>::from_error(error, RenderMode::PlainText, &pool))
  })
}

/// The title of `error`'s report.
fn title<E: FileforgeError>(error: E) -> String {
  let pool = DynamicDiagnosticPool::new();
  let report = format!("{:?}", RenderableError::<64, _, _>::from_error(error, RenderMode::PlainText, &pool));
  report.lines().find(|line| line.starts_with("i ")).unwrap_or("<no title>").trim().to_string()
}

type Reading<'a> = Sarc<'static, ProviderStream<&'a [u8]>>;
type Editing<'a, A = Preserve> = Sarc<'static, ProviderStream<&'a mut Vec<u8>>, A>;

async fn open(bytes: &[u8]) -> Reading<'_> {
  expect(BinaryReader::new_from_provider(bytes, Endianness::BigEndian, ReadHint::new()).into::<Sarc<_>>().await)
}

async fn open_mut(bytes: &mut Vec<u8>) -> Editing<'_> {
  expect(BinaryReader::new_from_provider(bytes, Endianness::BigEndian, ReadHint::new()).into::<Sarc<_>>().await)
}

/// Everything left in `stream`.
async fn rest<S: ReadableStream<Type = u8>>(stream: &mut S) -> Vec<u8> {
  let mut bytes = Vec::new();
  while let Ok(byte) = stream.next().await {
    bytes.push(byte);
  }
  bytes
}

async fn name_of<S: RestorableStream<Type = u8>, A: AlignmentPolicy>(sarc: &mut Sarc<'_, S, A>, entry: &SarcEntry) -> Option<String> {
  let mut name = expect(sarc.name(entry).await)?;
  Some(String::from_utf8(rest(&mut name).await).unwrap())
}

async fn data_of<S: RestorableStream<Type = u8>, A: AlignmentPolicy>(sarc: &mut Sarc<'_, S, A>, entry: &SarcEntry) -> Vec<u8> {
  let mut file = expect(sarc.file(entry).await);
  let data = rest(&mut file).await;
  assert_eq!(data.len() as u64, entry.len());
  data
}

/// Every file in the archive, by name, read through its entry.
async fn contents<S: RestorableStream<Type = u8>, A: AlignmentPolicy>(sarc: &mut Sarc<'_, S, A>) -> BTreeMap<String, Vec<u8>> {
  let mut files = BTreeMap::new();
  for index in 0..sarc.file_count() as u32 {
    let entry = expect(sarc.entry(index).await);
    let name = name_of(sarc, &entry).await.expect("every entry should have a name");
    files.insert(name, data_of(sarc, &entry).await);
  }
  files
}

/// Checks the archive, and that `find` gives each file of `expected` the right data.
async fn check<S: RestorableStream<Type = u8>, A: AlignmentPolicy>(sarc: &mut Sarc<'_, S, A>, expected: &BTreeMap<String, Vec<u8>>) {
  expect(sarc.validate().await);
  assert_eq!(&contents(sarc).await, expected);

  for (name, data) in expected {
    let entry = expect(sarc.find(name).await).unwrap_or_else(|| panic!("{name} should be found"));
    assert_eq!(&data_of(sarc, &entry).await, data, "{name}");
  }
}

// ---- Reading --------------------------------------------------------------------------------

#[test]
fn reads_every_file() {
  runtime().block_on(async {
    for bytes in [THREE_FILES_LE, THREE_FILES_BE] {
      check(&mut open(bytes).await, &common_files()).await;
    }

    let mut unicode = common_files();
    unicode.insert("Текст.txt".into(), b"unicode".to_vec());
    check(&mut open(UNICODE).await, &unicode).await;

    check(&mut open(COLLISIONS).await, &collision_files()).await;
  });
}

#[test]
fn a_missing_name_is_not_found() {
  runtime().block_on(async {
    let mut sarc = open(COLLISIONS).await;
    for name in ["", "Readme", "Readme.txt\n", "c/aycpvjtn", "Model/Actor.bfres2"] {
      assert!(expect(sarc.find(name).await).is_none(), "{name:?} shouldn't be found");
    }
  });
}

/// oead numbers every entry 1, even when entries share a hash, and sead uses the number to find
/// the first of them. So a sead-style lookup can miss a file in an archive oead wrote.
#[test]
fn collisions_as_oead_numbers_them() {
  runtime().block_on(async {
    let mut sarc = open(COLLISIONS_OEAD).await;

    // Both are found in the properly numbered archive (see `reads_every_file`), but here the
    // binary search lands on the second of the pair, and its number says it's the first.
    assert!(expect(sarc.find("c/lskfark").await).is_some());
    assert!(expect(sarc.find("c/gwbuwf").await).is_none());

    let error = sarc.validate().await.err().expect("the numbering should be reported");
    assert!(matches!(error, SarcError::WrongCollisionIndex { index: 1, expected: 2, found: 1 }));
  });
}

#[test]
fn reads_a_real_archive_through_yaz0() {
  runtime().block_on(async {
    let decompressed = decompress(SKY_WORLD);
    let expected = contents(&mut open(&decompressed).await).await;
    assert_eq!(expected.len(), 8);
    assert_eq!(expected["ScenarioInfo.byml"].len(), 240);

    let reader = BinaryReader::new_from_provider(SKY_WORLD, Endianness::BigEndian, ReadHint::new());
    let yaz0 = expect(reader.into_with::<Yaz0Stream<_, Immutable>>(Immutable).await);
    let mut sarc = expect(BinaryReader::new(yaz0, Endianness::BigEndian).into::<Sarc<_>>().await);

    // Just one file, found by name: reading everything through Yaz0 would be slow in debug builds.
    let entry = expect(sarc.find("ScenarioInfo.byml").await).unwrap();
    assert_eq!(data_of(&mut sarc, &entry).await, expected["ScenarioInfo.byml"]);
  });
}

#[test]
fn a_file_is_a_bounded_stream() {
  runtime().block_on(async {
    let mut sarc = open(THREE_FILES_LE).await;
    let entry = expect(sarc.find("Readme.txt").await).unwrap();
    let mut file = expect(sarc.file(&entry).await);

    assert_eq!(file.len(), Some(12));
    let start = file.snapshot();

    assert_eq!(expect(file.read(async |bytes: &[u8; 5]| *bytes).await), *b"hello");
    expect(file.skip(2).await);
    assert_eq!(expect(file.read(SINGLE).await), b's');

    // Reading or skipping past the end of the file fails, even though the archive goes on.
    assert!(file.read(async |bytes: &[u8; 5]| *bytes).await.is_err());
    assert!(file.skip(5).await.is_err());
    assert_eq!(file.offset(), 8);

    expect(file.restore(start).await);
    assert_eq!(rest(&mut file).await, b"hello, sarc\n");
  });
}

// ---- Damaged archives -----------------------------------------------------------------------

/// `bytes` with `patch` written at `offset`.
fn patched(bytes: &[u8], offset: usize, patch: &[u8]) -> Vec<u8> {
  let mut bytes = bytes.to_vec();
  bytes[offset..offset + patch.len()].copy_from_slice(patch);
  bytes
}

const ENTRY_0: usize = 0x20;

#[test]
fn rejects_damaged_entries() {
  runtime().block_on(async {
    assert!(matches!(open(THREE_FILES_LE).await.entry(3).await, Err(SarcError::NoSuchEntry { index: 3, file_count: 3 })));

    let end = u32::from_le_bytes(THREE_FILES_LE[ENTRY_0 + 12..ENTRY_0 + 16].try_into().unwrap());
    let end_before_start = patched(THREE_FILES_LE, ENTRY_0 + 8, &(end + 4).to_le_bytes());
    assert!(matches!(open(&end_before_start).await.entry(0).await, Err(SarcError::EndBeforeStart { index: 0, .. })));

    let outside = patched(THREE_FILES_LE, ENTRY_0 + 12, &0x10000u32.to_le_bytes());
    assert!(matches!(open(&outside).await.entry(0).await, Err(SarcError::DataOutsideArchive { index: 0, .. })));

    let name_outside = patched(THREE_FILES_LE, ENTRY_0 + 4, &0x0100_0100u32.to_le_bytes());
    assert!(matches!(open(&name_outside).await.entry(0).await, Err(SarcError::NameOutsideTable { index: 0, .. })));
  });
}

#[test]
fn reports_a_collision_number_that_reaches_before_the_table() {
  runtime().block_on(async {
    // Entry 1 is the second of a pair; say it's the third.
    let attributes = u32::from_le_bytes(COLLISIONS[ENTRY_0 + 16 + 4..ENTRY_0 + 16 + 8].try_into().unwrap());
    let damaged = patched(COLLISIONS, ENTRY_0 + 16 + 4, &((3 << 24) | (attributes & 0xFF_FFFF)).to_le_bytes());

    let error = open(&damaged).await.find("c/gwbuwf").await.err().expect("the collision number should be reported");
    assert!(matches!(error, SarcError::CollisionIndexTooLarge { index: 1, sequence: 3 }));
  });
}

#[test]
fn validate_reports_what_sead_relies_on() {
  runtime().block_on(async {
    // Swap two entries with different hashes.
    let mut swapped = COLLISIONS.to_vec();
    let (first, second) = (ENTRY_0 + 2 * 16, ENTRY_0 + 3 * 16);
    let entry = swapped[first..first + 16].to_vec();
    swapped.copy_within(second..second + 16, first);
    swapped[second..second + 16].copy_from_slice(&entry);
    assert!(matches!(open(&swapped).await.validate().await, Err(SarcError::NotSorted { index: 3 })));

    // Take the name away from the first of a pair.
    let unnamed = patched(COLLISIONS, ENTRY_0 + 4, &0u32.to_le_bytes());
    assert!(matches!(open(&unnamed).await.validate().await, Err(SarcError::UnnamedCollision { index: 1 })));

    // Change a byte of a name.
    let sarc_names = 0x20 + 7 * 16 + 8;
    let renamed = patched(COLLISIONS, sarc_names, b"X");
    assert!(matches!(open(&renamed).await.validate().await, Err(SarcError::NameHashMismatch { index: 0, .. })));
  });
}

#[test]
fn an_unnamed_entry_is_found_by_its_hash_alone() {
  runtime().block_on(async {
    let mut sarc = open(THREE_FILES_LE).await;
    let entry = expect(sarc.find("Readme.txt").await).unwrap();
    let position = ENTRY_0 + 16 * entry.index as usize + 4;

    let unnamed = patched(THREE_FILES_LE, position, &0u32.to_le_bytes());
    let mut sarc = open(&unnamed).await;

    let found = expect(sarc.find("Readme.txt").await).unwrap();
    assert_eq!(found.name, None);
    assert_eq!(data_of(&mut sarc, &found).await, b"hello, sarc\n");
    expect(sarc.validate().await);
  });
}

// ---- Editing --------------------------------------------------------------------------------

/// Replaces entry `index`'s data with `data`.
async fn set_data<S, A>(sarc: &mut Sarc<'_, S, A>, index: u32, data: &[u8])
where
  S: RestorableStream<Type = u8> + MutableStream + ResizableStream,
  A: AlignmentPolicy,
{
  let entry = expect(sarc.entry(index).await);
  let mut file = expect(sarc.file(&entry).await);
  expect(file.overwrite(entry.len(), []).await);
  insert(&mut file, data).await;
}

/// Inserts `data` at `stream`'s position.
async fn insert<S: ResizableStream<Type = u8>>(stream: &mut S, data: &[u8])
where
  S::OverwriteError: FileforgeError,
{
  for chunk in data.chunks(16) {
    match <[u8; 16]>::try_from(chunk) {
      Ok(chunk) => expect(stream.overwrite(0, chunk).await),
      Err(_) => {
        for &byte in chunk {
          expect(stream.overwrite(0, [byte]).await);
        }
      }
    }
  }
}

/// Where each file starts, from the start of the archive.
async fn positions<S: RestorableStream<Type = u8>, A: AlignmentPolicy>(sarc: &mut Sarc<'_, S, A>) -> BTreeMap<String, u64> {
  let mut positions = BTreeMap::new();
  for index in 0..sarc.file_count() as u32 {
    let entry = expect(sarc.entry(index).await);
    let name = name_of(sarc, &entry).await.unwrap();
    positions.insert(name, sarc.data_offset() + entry.start as u64);
  }
  positions
}

/// The alignment [`Preserve`] gives a file at `position`.
fn preserved(position: u64) -> u64 {
  if position == 0 {
    0x2000
  } else {
    (1u64 << position.trailing_zeros()).min(0x2000)
  }
}

/// Checks that every file in `before` that's still there keeps the alignment it had.
fn assert_alignment_kept(before: &BTreeMap<String, u64>, after: &BTreeMap<String, u64>) {
  for (name, &position) in before {
    if let Some(&moved) = after.get(name) {
      assert_eq!(moved % preserved(position), 0, "{name} moved from {position:#x} to {moved:#x}");
    }
  }
}

#[test]
fn growing_and_shrinking_a_file_keeps_the_others() {
  runtime().block_on(async {
    for (name, grow_by) in [("Model/Actor.bfres", 1), ("Param/Actor.byml", 37), ("Readme.txt", 200), ("Model/Actor.bfres", 0x2100)] {
      let mut bytes = THREE_FILES_LE.to_vec();
      let mut expected = common_files();
      let mut sarc = open_mut(&mut bytes).await;
      let before = positions(&mut sarc).await;

      let entry = expect(sarc.find(name).await).unwrap();
      let grown: Vec<u8> = (0..entry.len() + grow_by).map(|i| (i * 7) as u8).collect();
      set_data(&mut sarc, entry.index, &grown).await;
      expected.insert(name.into(), grown);

      check(&mut sarc, &expected).await;
      assert_alignment_kept(&before, &positions(&mut sarc).await);

      // And back down to a few bytes.
      let entry = expect(sarc.find(name).await).unwrap();
      set_data(&mut sarc, entry.index, b"tiny").await;
      expected.insert(name.into(), b"tiny".to_vec());

      check(&mut sarc, &expected).await;
      assert_alignment_kept(&before, &positions(&mut sarc).await);
      drop(sarc);

      oead_agrees(&bytes, &expected);
    }
  });
}

#[test]
fn adding_and_removing_files() {
  runtime().block_on(async {
    let mut bytes = COLLISIONS.to_vec();
    let mut expected = collision_files();
    let mut sarc = open_mut(&mut bytes).await;

    // A file whose hash collides with nothing, then one that joins a colliding pair's hash.
    for (name, data) in [("New/file.txt", &b"brand new"[..]), ("Empty.bin", &b""[..])] {
      let before = positions(&mut sarc).await;
      let entry = expect(sarc.add_file(name).await);
      set_data(&mut sarc, entry.index, data).await;
      expected.insert(name.into(), data.to_vec());

      check(&mut sarc, &expected).await;
      assert_alignment_kept(&before, &positions(&mut sarc).await);
    }

    // Names that can't be added.
    assert!(matches!(sarc.add_file("Readme.txt").await, Err(SarcEditError::NameExists { .. })));
    assert!(matches!(sarc.add_file("").await, Err(SarcEditError::InvalidName)));
    assert!(matches!(sarc.add_file("a\0b").await, Err(SarcEditError::InvalidName)));

    // Remove one of each colliding pair, then everything else.
    for name in ["c/aycpvjtnn", "c/lskfark", "Model/Actor.bfres", "New/file.txt", "c/kevbqeqbp", "Empty.bin", "c/gwbuwf", "Param/Actor.byml", "Readme.txt"] {
      let before = positions(&mut sarc).await;
      let entry = expect(sarc.find(name).await).unwrap();
      expect(sarc.remove_file(entry.index).await);
      expected.remove(name);

      check(&mut sarc, &expected).await;
      assert!(expect(sarc.find(name).await).is_none(), "{name} should be gone");
      assert_alignment_kept(&before, &positions(&mut sarc).await);
    }

    assert_eq!(sarc.file_count(), 0);
    drop(sarc);
    oead_agrees(&bytes, &expected);
  });
}

#[test]
fn adding_a_file_that_collides() {
  runtime().block_on(async {
    let mut bytes = THREE_FILES_LE.to_vec();
    let mut expected = common_files();
    let mut sarc = open_mut(&mut bytes).await;

    // Add each of a colliding pair: the second must be numbered 2.
    for name in ["c/aycpvjtnn", "c/kevbqeqbp"] {
      let entry = expect(sarc.add_file(name).await);
      set_data(&mut sarc, entry.index, name.as_bytes()).await;
      expected.insert(name.into(), name.as_bytes().to_vec());
    }

    check(&mut sarc, &expected).await;
    let second = expect(sarc.find("c/kevbqeqbp").await).unwrap();
    assert_eq!(second.name.unwrap().sequence.get(), 2);
  });
}

#[test]
fn creating_an_archive() {
  runtime().block_on(async {
    for endianness in [Endianness::LittleEndian, Endianness::BigEndian] {
      let mut bytes = Vec::new();
      let reader = BinaryReader::new_from_provider(&mut bytes, Endianness::BigEndian, ReadHint::new());
      let mut sarc = expect(Sarc::create(reader, endianness, Preserve::default()).await);
      let mut expected = BTreeMap::new();

      for (name, data) in collision_files() {
        let entry = expect(sarc.add_file(&name).await);
        set_data(&mut sarc, entry.index, &data).await;
        expected.insert(name, data);
      }

      check(&mut sarc, &expected).await;
      assert_eq!(sarc.header().endianness, endianness);
      drop(sarc);

      // Opened afresh, too.
      check(&mut open(&bytes).await, &expected).await;
      oead_agrees(&bytes, &expected);
    }
  });
}

#[test]
fn realigning() {
  runtime().block_on(async {
    let mut bytes = THREE_FILES_LE.to_vec();
    let expected = common_files();
    let sarc = open_mut(&mut bytes).await;

    // Preserve keeps what's there, so realigning to it changes nothing.
    let mut sarc = sarc;
    let size = sarc.header().file_size;
    expect(sarc.realign().await);
    assert_eq!(sarc.header().file_size, size);

    let mut sarc: Editing<'_, Fixed> = sarc.with_policy(Fixed(0x100));
    expect(sarc.realign().await);
    check(&mut sarc, &expected).await;

    for (name, position) in positions(&mut sarc).await {
      assert_eq!(position % 0x100, 0, "{name} is at {position:#x}");
    }

    // Already aligned: nothing moves.
    let size = sarc.header().file_size;
    expect(sarc.realign().await);
    assert_eq!(sarc.header().file_size, size);

    // Growing a file into the padding after it doesn't move the next one; growing past it moves
    // the next one by a multiple of 0x100.
    let before = positions(&mut sarc).await;
    let first = before.iter().min_by_key(|(_, &position)| position).unwrap().0.clone();
    let entry = expect(sarc.find(&first).await).unwrap();
    let mut grown = expected[&first].clone();
    grown.extend([1; 10]);
    set_data(&mut sarc, entry.index, &grown).await;

    let after = positions(&mut sarc).await;
    for (name, position) in &before {
      if *name != first {
        assert_eq!(after[name], *position, "{name} shouldn't move");
      }
    }

    let entry = expect(sarc.find(&first).await).unwrap();
    grown.extend([2; 0x150]);
    set_data(&mut sarc, entry.index, &grown).await;

    for (name, position) in positions(&mut sarc).await {
      assert_eq!(position % 0x100, 0, "{name} is at {position:#x}");
    }

    let mut expected = expected;
    expected.insert(first, grown);
    check(&mut sarc, &expected).await;
    drop(sarc);
    oead_agrees(&bytes, &expected);
  });
}

#[test]
fn editing_a_real_archive_through_yaz0() {
  runtime().block_on(async {
    let mut expected = contents(&mut open(&decompress(SKY_WORLD)).await).await;

    let mut compressed = SKY_WORLD.to_vec();
    {
      let reader = BinaryReader::new_from_provider(&mut compressed, Endianness::BigEndian, ReadHint::new());
      let yaz0 = expect(reader.into_with::<Yaz0Stream<_, Mutable>>(Mutable).await);
      let mut sarc = expect(BinaryReader::new(yaz0, Endianness::BigEndian).into::<Sarc<_>>().await);

      let entry = expect(sarc.find("ScenarioInfo.byml").await).unwrap();
      let mut file = expect(sarc.file(&entry).await);
      expect(file.overwrite(0, *b"HELLO").await);
    }

    expected.get_mut("ScenarioInfo.byml").unwrap().splice(0..0, *b"HELLO");

    let decompressed = decompress(&compressed);
    let mut sarc = open(&decompressed).await;
    check(&mut sarc, &expected).await;
    oead_agrees(&decompressed, &expected);
  });
}

/// Random edits, checked after each one against a model of what the archive should hold.
/// `SARC_SEEDS` sets how many runs of 60 edits to make (4 by default). Every third run edits a
/// big-endian archive, and every other one realigns to `Fixed(0x40)` first and keeps to it.
#[test]
fn random_edits() {
  let seeds: u64 = std::env::var("SARC_SEEDS").ok().and_then(|seeds| seeds.parse().ok()).unwrap_or(4);

  runtime().block_on(async {
    for seed in 1..=seeds {
      let (mut bytes, expected) = match seed % 3 {
        0 => (THREE_FILES_BE.to_vec(), common_files()),
        _ => (COLLISIONS.to_vec(), collision_files()),
      };

      let sarc = open_mut(&mut bytes).await;

      let expected = if seed % 2 == 0 {
        let mut sarc = sarc.with_policy(Fixed(0x40));
        expect(sarc.realign().await);
        random_run(seed, &mut sarc, expected, |_, after| {
          for (name, position) in after {
            assert_eq!(position % 0x40, 0, "{name} is at {position:#x}");
          }
        })
        .await
      } else {
        let mut sarc = sarc;
        random_run(seed, &mut sarc, expected, assert_alignment_kept).await
      };

      oead_agrees(&bytes, &expected);
    }
  });
}

/// Makes 60 random edits to `sarc`, checking it and `aligned(before, after)` after each, and
/// returns what it should hold.
async fn random_run<A: AlignmentPolicy>(
  seed: u64,
  sarc: &mut Editing<'_, A>,
  mut expected: BTreeMap<String, Vec<u8>>,
  aligned: impl Fn(&BTreeMap<String, u64>, &BTreeMap<String, u64>),
) -> BTreeMap<String, Vec<u8>> {
  let names: Vec<String> = ["c/aycpvjtnn", "c/kevbqeqbp", "c/gwbuwf", "c/lskfark", "a", "b.bin", "Model/Actor.bfres", "Param/Actor.byml", "Readme.txt", "Текст.txt", "deep/er/path/name.bntx"]
    .into_iter()
    .map(String::from)
    .collect();

  let mut random = Random(seed);

  for step in 0..60 {
    let before = positions(sarc).await;
    let files: Vec<String> = expected.keys().cloned().collect();
    let what;

    match random.below(6) {
      0 => {
        let name = &names[random.below(names.len() as u64) as usize];
        what = format!("add {name}");
        match sarc.add_file(name).await {
          Ok(entry) => {
            let data = random.bytes(40);
            set_data(sarc, entry.index, &data).await;
            expected.insert(name.clone(), data);
          }
          Err(SarcEditError::NameExists { .. }) => assert!(expected.contains_key(name)),
          Err(error) => panic!("seed {seed}, step {step} ({what}): {}", title(error)),
        }
      }
      _ if files.is_empty() => what = "nothing".into(),
      1 => {
        let name = &files[random.below(files.len() as u64) as usize];
        what = format!("remove {name}");
        let entry = expect(sarc.find(name).await).unwrap();
        expect(sarc.remove_file(entry.index).await);
        expected.remove(name);
      }
      operation => {
        let name = &files[random.below(files.len() as u64) as usize];
        let data = expected.get_mut(name).unwrap();
        let entry = expect(sarc.find(name).await).unwrap();
        let at = random.below(data.len() as u64 + 1) as usize;
        let length = random.below((data.len() - at) as u64 + 1) as usize;
        let mut file = expect(sarc.file(&entry).await);
        expect(file.skip(at as u64).await);

        match operation {
          2 => {
            let new = random.bytes(30);
            what = format!("insert {} bytes at {at} in {name}", new.len());
            insert(&mut file, &new).await;
            data.splice(at..at, new);
          }
          3 => {
            what = format!("delete {length} bytes at {at} in {name}");
            expect(file.overwrite(length as u64, []).await);
            data.drain(at..at + length);
          }
          4 => {
            what = format!("replace {length} bytes at {at} in {name} with 3");
            let new = [random.below(256) as u8; 3];
            expect(file.overwrite(length as u64, new).await);
            data.splice(at..at + length, new);
          }
          _ => {
            what = format!("change a byte at {at} in {name}");
            if at < data.len() {
              let new = random.below(256) as u8;
              expect(file.mutate(async |byte: &mut [u8; 1]| byte[0] = new).await);
              data[at] = new;
            }
          }
        }
      }
    }

    let result = std::panic::AssertUnwindSafe(async {
      check(sarc, &expected).await;
      aligned(&before, &positions(sarc).await);
    });
    if let Err(panic) = futures_catch(result).await {
      panic!("seed {seed}, step {step} ({what}): {panic}");
    }
  }

  expected
}

// ---- Helpers --------------------------------------------------------------------------------

/// Runs `future`, turning a panic into an error message.
async fn futures_catch<F: std::future::Future<Output = ()>>(future: std::panic::AssertUnwindSafe<F>) -> Result<(), String> {
  let mut future = std::pin::pin!(future.0);
  std::future::poll_fn(|context| {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| future.as_mut().poll(context))) {
      Ok(std::task::Poll::Ready(())) => std::task::Poll::Ready(Ok(())),
      Ok(std::task::Poll::Pending) => std::task::Poll::Pending,
      Err(payload) => std::task::Poll::Ready(Err(payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|message| message.to_string()))
        .unwrap_or_default())),
    }
  })
  .await
}

/// A small, deterministic xorshift generator.
struct Random(u64);

impl Random {
  fn next(&mut self) -> u64 {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    self.0
  }

  fn below(&mut self, bound: u64) -> u64 {
    self.next() % bound.max(1)
  }

  fn bytes(&mut self, max: u64) -> Vec<u8> {
    let length = self.below(max + 1);
    (0..length).map(|_| self.next() as u8).collect()
  }
}

fn decompress(bytes: &[u8]) -> Vec<u8> {
  yaz0::inflate::Yaz0Archive::new(Cursor::new(bytes)).and_then(|mut archive| archive.decompress()).expect("the referee should decompress it")
}

/// Has oead read `archive`, if `OEAD_PYTHON` names a Python with it, and checks it finds `expected`.
fn oead_agrees(archive: &[u8], expected: &BTreeMap<String, Vec<u8>>) {
  let Ok(python) = std::env::var("OEAD_PYTHON") else { return };

  // Tests run in parallel, so each call gets its own file.
  static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
  let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  let path = std::env::temp_dir().join(format!("fileforge-sarc-{}-{call}.sarc", std::process::id()));
  std::fs::write(&path, archive).unwrap();

  let script = "import oead, sys\nfor f in oead.Sarc(open(sys.argv[1], 'rb').read()).get_files(): print(f.name.encode('utf-8').hex(), bytes(f.data).hex())";
  let output = std::process::Command::new(python).args(["-c", script]).arg(&path).output().expect("OEAD_PYTHON should run");
  std::fs::remove_file(&path).ok();
  assert!(output.status.success(), "oead failed: {}", String::from_utf8_lossy(&output.stderr));

  let hex = |text: &str| (0..text.len()).step_by(2).map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap()).collect::<Vec<u8>>();
  let found: BTreeMap<String, Vec<u8>> = String::from_utf8(output.stdout)
    .unwrap()
    .lines()
    .map(|line| {
      let (name, data) = line.split_once(' ').unwrap_or((line, ""));
      (String::from_utf8(hex(name)).unwrap(), hex(data))
    })
    .collect();

  assert_eq!(&found, expected, "oead reads something else");
}
