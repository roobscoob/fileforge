//! End-to-end tests for editing Yaz0-compressed data in place.
//!
//! Each case compresses some data with the independent `yaz0` crate, edits it through
//! fileforge's `Yaz0Stream` (inserting, replacing or deleting bytes at a position), then
//! decompresses the result with the `yaz0` crate again and compares it with the expected bytes.
//! Using a separate implementation as the referee means a bug shared by fileforge's encoder and
//! decoder can't hide itself.
//!
//! Every edit also checks `overwrite`'s contract: afterwards the stream sits just after the new
//! data, `len()` is the edited size, and reading on gives the bytes that follow. At odd positions
//! a second edit is made on the same stream.
//!
//! The sweeps marked `#[ignore]` take minutes; run them with `cargo test --release -p
//! fileforge-test --test yaz0_editing -- --ignored --nocapture`.

use std::{
  cell::{Cell, RefCell},
  collections::BTreeMap,
  io::Cursor,
  panic::{self, AssertUnwindSafe},
  sync::Once,
};

use fileforge::{
  binary_reader::{endianness::Endianness, BinaryReader},
  diagnostic::pool::dynamic::DynamicDiagnosticPool,
  error::{render::buffer::cell::tag::context::RenderMode, FileforgeError, RenderableError},
  provider::hint::ReadHint,
  stream::{ReadableStream, ResizableStream},
};
use fileforge_nintendo::sead::yaz0::{readable::Mutable, Yaz0Stream};
use yaz0::CompressionLevel;

const LOOKAHEAD: CompressionLevel = CompressionLevel::Lookahead { quality: 10 };
const NAIVE: CompressionLevel = CompressionLevel::Naive { quality: 3 };

fn compress(data: &[u8], level: CompressionLevel) -> Vec<u8> {
  let mut out = Vec::new();
  yaz0::Yaz0Writer::new(&mut out).compress_and_write(data, level).unwrap();
  out
}

fn decompress(bytes: &[u8]) -> Result<Vec<u8>, String> {
  yaz0::inflate::Yaz0Archive::new(Cursor::new(bytes)).and_then(|mut archive| archive.decompress()).map_err(|error| format!("{error:?}"))
}

/// The title of an error's report.
fn describe<E: FileforgeError>(error: E) -> String {
  let pool = DynamicDiagnosticPool::new();
  let report = format!("{:?}", RenderableError::<64, _, _>::from_error(error, RenderMode::PlainText, &pool));
  report.lines().find(|line| line.starts_with("i ")).unwrap_or("<no title>").trim().to_string()
}

/// A Yaz0 stream by hand: the header, then `groups` of (header byte, operation bytes).
fn yaz0_by_hand(decompressed_size: usize, groups: &[(u8, &[u8])]) -> Vec<u8> {
  let mut out = [&b"Yaz0"[..], &(decompressed_size as u32).to_be_bytes(), &[0; 8]].concat();
  for (header, operations) in groups {
    out.push(*header);
    out.extend(*operations);
  }
  out
}

// ---- Data ----------------------------------------------------------------------------------------

/// Text with plenty of repetition, so the compressor uses back-references, including far ones.
fn text(length: usize) -> Vec<u8> {
  let mut out = Vec::new();
  let mut line = 0u32;

  while out.len() < length {
    out.extend(format!("entry {}: the value is {} and the name is item_{}\n", line % 23, (line * 7) % 101, line % 5).as_bytes());
    line += 1;
  }

  out.truncate(length);
  out
}

/// Bytes that barely compress.
fn noise(length: usize, seed: u32) -> Vec<u8> {
  let mut x = seed;
  (0..length)
    .map(|_| {
      x ^= x << 13;
      x ^= x >> 17;
      x ^= x << 5;
      x as u8
    })
    .collect()
}

/// Runs of one byte up to 300 long, which compress to back-references that overlap themselves.
fn runs(length: usize) -> Vec<u8> {
  let mut out = Vec::new();
  let mut i = 0u32;
  while out.len() < length {
    let run = 1 + (i * 37 % 300) as usize;
    out.extend(std::iter::repeat(b"abcAB"[(i % 5) as usize]).take(run));
    out.extend(format!("{i}").as_bytes());
    i += 1;
  }
  out.truncate(length);
  out
}

/// Stretches of text, noise and runs.
fn mixed(length: usize) -> Vec<u8> {
  let mut out = Vec::new();
  let mut i = 0u32;
  while out.len() < length {
    match i % 3 {
      0 => out.extend(text(200 + (i as usize * 13) % 500)),
      1 => out.extend(noise(50 + (i as usize * 7) % 200, i + 1)),
      _ => out.extend(runs(100 + (i as usize * 11) % 400)),
    }
    i += 1;
  }
  out.truncate(length);
  out
}

/// The same 4 KB of noise three times: after the first copy, every back-reference reaches exactly
/// 4096 bytes back, the furthest Yaz0 allows.
fn repeated_noise() -> Vec<u8> {
  let n = noise(4096, 3);
  [&n[..], &n[..], &n[..]].concat()
}

// ---- Edits ---------------------------------------------------------------------------------------

struct Edit {
  name: &'static str,
  replaced: usize,
  inserted: Vec<u8>,
}

impl Edit {
  fn new(name: &'static str, replaced: usize, inserted: impl Into<Vec<u8>>) -> Self {
    Edit { name, replaced, inserted: inserted.into() }
  }
}

fn small_edits() -> Vec<Edit> {
  vec![
    Edit::new("Insert", 0, *b"XY"),
    Edit::new("ReplaceOne", 1, *b"Z"),
    Edit::new("ShrinkFourToTwo", 4, *b"XY"),
    Edit::new("Delete", 3, []),
  ]
}

fn medium_edits() -> Vec<Edit> {
  vec![
    Edit::new("Insert40", 0, [b'#'; 40]),
    Edit::new("Replace30With50", 30, (0..50u8).map(|i| b'a' + i % 26).collect::<Vec<_>>()),
    Edit::new("Delete100", 100, []),
    Edit::new("Replace200With3", 200, *b"abc"),
  ]
}

/// Edits of thousands of bytes, most around the 4096-byte history limit.
fn large_edits(data: &[u8]) -> Vec<Edit> {
  vec![
    Edit::new("Insert1000Text", 0, text(1_500)[500..].to_vec()),
    Edit::new("Insert4096Noise", 0, noise(4_096, 99)),
    Edit::new("Insert5000Text", 0, text(5_500)[500..].to_vec()),
    Edit::new("Insert10000Noise", 0, noise(10_000, 123)),
    Edit::new("Delete1000", 1_000, []),
    Edit::new("Delete4096", 4_096, []),
    Edit::new("Delete4097", 4_097, []),
    Edit::new("Delete5000", 5_000, []),
    Edit::new("Replace5000With10", 5_000, noise(10, 5)),
    Edit::new("Replace10WithDataStart5000", 10, data[..5_000].to_vec()),
    Edit::new("Replace4096With4096Text", 4_096, text(4_596)[500..].to_vec()),
    Edit::new("Replace6000With4097Noise", 6_000, noise(4_097, 77)),
  ]
}

/// Opens `compressed`, moves to `at` and replaces `replaced` bytes with `inserted` (then, with
/// `second_insert`, inserts "!" with a second `overwrite`), checking `overwrite`'s contract.
async fn edit(compressed: &mut Vec<u8>, at: usize, replaced: usize, inserted: &[u8], second_insert: bool) -> Result<(), String> {
  match inserted.len() {
    0 => edit_sized::<0>(compressed, at, replaced, inserted, second_insert).await,
    1 => edit_sized::<1>(compressed, at, replaced, inserted, second_insert).await,
    2 => edit_sized::<2>(compressed, at, replaced, inserted, second_insert).await,
    3 => edit_sized::<3>(compressed, at, replaced, inserted, second_insert).await,
    10 => edit_sized::<10>(compressed, at, replaced, inserted, second_insert).await,
    40 => edit_sized::<40>(compressed, at, replaced, inserted, second_insert).await,
    50 => edit_sized::<50>(compressed, at, replaced, inserted, second_insert).await,
    1_000 => edit_sized::<1_000>(compressed, at, replaced, inserted, second_insert).await,
    4_096 => edit_sized::<4_096>(compressed, at, replaced, inserted, second_insert).await,
    4_097 => edit_sized::<4_097>(compressed, at, replaced, inserted, second_insert).await,
    5_000 => edit_sized::<5_000>(compressed, at, replaced, inserted, second_insert).await,
    10_000 => edit_sized::<10_000>(compressed, at, replaced, inserted, second_insert).await,
    other => panic!("add an `edit_sized::<{other}>` arm"),
  }
}

async fn edit_sized<const N: usize>(compressed: &mut Vec<u8>, at: usize, replaced: usize, inserted: &[u8], second_insert: bool) -> Result<(), String> {
  let following = decompress(compressed).map_err(|error| format!("the referee can't read the original: {error}"))?[at + replaced..].to_vec();
  let data: [u8; N] = inserted.try_into().unwrap();

  let reader = BinaryReader::new_from_provider(compressed, Endianness::BigEndian, ReadHint::new());
  let mut stream = reader.into_with::<Yaz0Stream<_, Mutable>>(Mutable).await.map_err(|error| format!("opening failed: {}", describe(error)))?;
  stream.skip(at as u64).await.map_err(|error| format!("moving to the edit failed: {}", describe(error)))?;
  stream.overwrite(replaced as u64, data).await.map_err(|error| format!("the edit failed: {}", describe(error)))?;

  let mut lands = (at + N) as u64;
  if stream.offset() != lands {
    return Err("contract: the stream doesn't land after the new data".into());
  }

  if second_insert {
    stream.overwrite(0, *b"!").await.map_err(|error| format!("the second edit failed: {}", describe(error)))?;
    lands += 1;
    if stream.offset() != lands {
      return Err("contract: after a second edit, the stream doesn't land after its data".into());
    }
  }

  if stream.len() != Some(lands + following.len() as u64) {
    return Err("contract: len() isn't the edited size".into());
  }

  for &expected in following.iter().take(16) {
    let actual = stream
      .read::<1, _>(async |bytes: &[u8; 1]| bytes[0])
      .await
      .map_err(|error| format!("contract: reading on after the edit failed: {}", describe(error)))?;
    if actual != expected {
      return Err("contract: reading on after the edit gives the wrong bytes".into());
    }
  }

  Ok(())
}

// ---- Running cases -------------------------------------------------------------------------------

thread_local! {
  /// Set while a case runs, so the panic hook records the panic instead of printing it.
  static CATCHING: Cell<bool> = const { Cell::new(false) };
  static PANIC_LOCATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn install_panic_hook() {
  static ONCE: Once = Once::new();
  ONCE.call_once(|| {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
      if CATCHING.with(Cell::get) {
        PANIC_LOCATION.with(|location| *location.borrow_mut() = info.location().map(|location| format!("{}:{}", location.file(), location.line())));
      } else {
        default_hook(info);
      }
    }));
  });
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
  payload
    .downcast_ref::<&str>()
    .map(|message| message.to_string())
    .or_else(|| payload.downcast_ref::<String>().cloned())
    .unwrap_or_else(|| "<non-string panic>".into())
}

/// How each case turned out, grouped by outcome, with the first example of each.
struct Outcomes {
  runtime: tokio::runtime::Runtime,
  outcomes: BTreeMap<String, (usize, String)>,
  total: usize,
}

impl Outcomes {
  fn new() -> Self {
    install_panic_hook();
    Outcomes {
      runtime: tokio::runtime::Builder::new_current_thread().build().unwrap(),
      outcomes: BTreeMap::new(),
      total: 0,
    }
  }

  /// Edits `compressed` (which decompresses to `data`), with a second insert at odd positions.
  fn run(&mut self, label: &str, compressed: &[u8], data: &[u8], at: usize, case: &Edit) {
    if at + case.replaced > data.len() {
      return;
    }

    let second_insert = at % 2 == 1;
    let second: &[u8] = if second_insert { b"!" } else { b"" };
    let expected = [&data[..at], &case.inserted, second, &data[at + case.replaced..]].concat();
    let mut edited = compressed.to_vec();

    CATCHING.with(|catching| catching.set(true));
    let result = panic::catch_unwind(AssertUnwindSafe(|| self.runtime.block_on(edit(&mut edited, at, case.replaced, &case.inserted, second_insert))));
    // The referee can itself panic on corrupt input, so guard it too.
    let decompressed = result.as_ref().is_ok_and(|result| result.is_ok()).then(|| panic::catch_unwind(|| decompress(&edited)));
    CATCHING.with(|catching| catching.set(false));

    let outcome = match (result, decompressed) {
      (Err(payload), _) => {
        let location = PANIC_LOCATION.with(|location| location.borrow_mut().take()).unwrap_or_default();
        // Group by message and place, not by the value in the message.
        let message = panic_message(&*payload);
        let message = message.split(" on an `Err` value").next().unwrap_or_default().to_string();
        let location = location.split("fileforge-nintendo").last().unwrap_or_default().replace('\\', "/");
        format!("panic: {message} at fileforge-nintendo{location}")
      }
      (Ok(Err(error)), _) => format!("error: {error}"),
      (Ok(Ok(())), Some(Err(_))) => "wrong output: crashes the referee".into(),
      (Ok(Ok(())), Some(Ok(Err(error)))) => format!("wrong output: doesn't decompress ({error})"),
      (Ok(Ok(())), Some(Ok(Ok(actual)))) if actual == expected => "ok".into(),
      (Ok(Ok(())), Some(Ok(Ok(actual)))) if actual.len() == expected.len() => "wrong output: right length, wrong bytes".into(),
      (Ok(Ok(())), _) => "wrong output: wrong length".into(),
    };

    self.total += 1;
    let entry = self.outcomes.entry(outcome).or_insert((0, String::new()));
    entry.0 += 1;
    if entry.1.is_empty() {
      entry.1 = format!("{label}, {} at {at}", case.name);
    }
  }

  /// Runs `edits` at each of `positions` on `data`, compressed at `level`.
  fn sweep(&mut self, label: &str, data: &[u8], level: CompressionLevel, edits: &[Edit], positions: impl IntoIterator<Item = usize>) {
    let compressed = compress(data, level);
    assert_eq!(decompress(&compressed).as_deref(), Ok(data), "the referee should round-trip {label} unedited");

    for at in positions {
      for case in edits {
        self.run(label, &compressed, data, at, case);
      }
    }
  }

  fn assert_all_ok(self) {
    let summary: String = self.outcomes.iter().map(|(outcome, (count, example))| format!("{count:>6}  {outcome}\n        e.g. {example}\n")).collect();
    println!("{} edits:\n{summary}", self.total);

    let failures: usize = self.outcomes.iter().filter(|(outcome, _)| *outcome != "ok").map(|(_, (count, _))| count).sum();
    assert_eq!(failures, 0, "{failures} of {} edits failed:\n{summary}", self.total);
  }
}

// ---- Tests ---------------------------------------------------------------------------------------

#[test]
fn edits_round_trip() {
  let mut outcomes = Outcomes::new();

  let short = b"The quick brown fox jumps over the lazy dog. The quick brown fox jumps again!";
  outcomes.sweep("short text", short, LOOKAHEAD, &small_edits(), 0..=short.len());

  // Sparse: every edit re-encodes up to 4 KB, which is slow in a debug build. The sweeps below
  // cover far more.
  let long = text(12_000);
  outcomes.sweep("long text", &long, LOOKAHEAD, &small_edits(), (0..=long.len()).step_by(997));

  outcomes.assert_all_ok();
}

/// A back-reference 4096 bytes back that starts 4095 bytes after the edit reads a byte from
/// before it, so the edit has to re-encode it.
#[test]
fn a_back_reference_reaching_the_edit_from_4096_bytes_on_is_repaired() {
  let r: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
  let data = [&r[..], &r[..10]].concat();

  // 512 groups of 8 literals, then "copy 10 bytes from 4096 back".
  let groups: Vec<(u8, &[u8])> = r.chunks(8).map(|chunk| (0xFF, chunk)).chain([(0x00, &[0x8F, 0xFF][..])]).collect();
  let compressed = yaz0_by_hand(data.len(), &groups);
  assert_eq!(decompress(&compressed).unwrap(), data);

  let mut outcomes = Outcomes::new();
  outcomes.run("by hand", &compressed, &data, 0, &Edit::new("ReplaceOne", 1, *b"Z"));
  outcomes.run("by hand", &compressed, &data, 2, &Edit::new("Insert", 0, *b"XY"));
  outcomes.run("by hand", &compressed, &data, 0, &Edit::new("DeleteOne", 1, []));
  outcomes.assert_all_ok();
}

#[test]
fn swapping_one_literal_for_another_keeps_the_size() {
  let data: Vec<u8> = (b'a'..b'a' + 16).collect();
  let compressed = yaz0_by_hand(16, &[(0xFF, &data[..8]), (0xFF, &data[8..])]);

  let mut edited = compressed.clone();
  let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();
  runtime.block_on(edit(&mut edited, 15, 1, b"Z", false)).unwrap();

  assert_eq!(decompress(&edited).unwrap(), [&data[..15], b"Z"].concat());
  assert_eq!(edited.len(), compressed.len());
}

#[test]
fn an_edit_past_the_end_fails_without_changing_anything() {
  let data = text(9_000);
  let compressed = compress(&data, LOOKAHEAD);
  let runtime = tokio::runtime::Builder::new_current_thread().build().unwrap();

  for at in [0, 1, 100, 4_000, 8_990, 9_000] {
    for past in [1, 5, 10_000] {
      let length = (data.len() - at) as u64 + past;
      let mut edited = compressed.clone();

      let result = runtime.block_on(async {
        let reader = BinaryReader::new_from_provider(&mut edited, Endianness::BigEndian, ReadHint::new());
        let mut stream = reader.into_with::<Yaz0Stream<_, Mutable>>(Mutable).await.map_err(describe)?;
        stream.skip(at as u64).await.map_err(describe)?;
        stream.overwrite(length, *b"XY").await.map_err(describe)
      });

      assert!(result.is_err(), "overwrite({length}) at {at} should fail");
      assert!(edited == compressed, "overwrite({length}) at {at} changed the file");
    }
  }
}

#[test]
#[ignore = "takes about 10 minutes in release"]
fn wide_sweep() {
  let mut outcomes = Outcomes::new();

  for (name, data) in [("text", text(9_000)), ("runs", runs(9_000)), ("noise", noise(3_000, 7)), ("mixed", mixed(12_000))] {
    for (level_name, level) in [("lookahead", LOOKAHEAD), ("naive", NAIVE)] {
      let label = format!("{name}/{level_name}");
      outcomes.sweep(&label, &data, level, &small_edits(), (0..=data.len()).step_by(3));
      outcomes.sweep(&label, &data, level, &medium_edits(), (0..=data.len()).step_by(15));
    }
  }

  outcomes.assert_all_ok();
}

#[test]
#[ignore = "takes about 20 minutes in release"]
fn large_edit_sweep() {
  let mut outcomes = Outcomes::new();

  for (name, data) in [("text", text(20_000)), ("runs", runs(20_000)), ("noise", noise(8_000, 7)), ("mixed", mixed(24_000))] {
    for (level_name, level) in [("lookahead", LOOKAHEAD), ("naive", NAIVE)] {
      for case in large_edits(&data) {
        // Every 97th position, plus the very end.
        let end = data.len() - case.replaced.min(data.len());
        outcomes.sweep(&format!("{name}/{level_name}"), &data, level, &[case], (0..=data.len()).step_by(97).chain([end]));
      }
    }
  }

  outcomes.assert_all_ok();
}

#[test]
#[ignore = "takes about 5 minutes in release"]
fn repeated_noise_sweep() {
  let data = repeated_noise();
  let positions = (0..=data.len()).step_by(13).chain(0..600).chain(4096 - 300..4096 + 600);

  let mut outcomes = Outcomes::new();
  outcomes.sweep("repeated noise", &data, LOOKAHEAD, &small_edits(), positions);
  outcomes.assert_all_ok();
}

/// fileforge's own encoder picks the furthest match where the `yaz0` crate picks the nearest, so
/// data fileforge has already edited has back-references the other sweeps rarely see.
#[test]
#[ignore = "takes about 7 minutes in release"]
fn re_edit_sweep() {
  let mut outcomes = Outcomes::new();

  for (name, data) in [("text", text(9_000)), ("runs", runs(9_000)), ("noise", noise(5_000, 7)), ("repeated noise", repeated_noise())] {
    let compressed = compress(&data, LOOKAHEAD);

    for first_at in [0, 700, 2_500] {
      let first = [b'q'; 1_000];
      let mut once = compressed.clone();
      outcomes.runtime.block_on(edit(&mut once, first_at, 0, &first, false)).unwrap();

      let data_once = [&data[..first_at], &first[..], &data[first_at..]].concat();
      assert_eq!(decompress(&once).unwrap(), data_once, "{name}: the first edit, at {first_at}, should be right");

      let label = format!("{name} after inserting at {first_at}");
      for at in (0..=data_once.len()).step_by(29) {
        for case in small_edits() {
          outcomes.run(&label, &once, &data_once, at, &case);
        }
      }
    }
  }

  outcomes.assert_all_ok();
}
