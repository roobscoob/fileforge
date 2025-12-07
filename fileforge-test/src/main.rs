use std::{io::Cursor, time::Instant};

use fileforge::{
  binary_reader::{diagnostic_store::DiagnosticKind, endianness::Endianness, BinaryReader},
  diagnostic::{
    node::branch::DiagnosticBranch,
    pool::{
      dynamic::DynamicDiagnosticPool,
      fixed::{entry::FixedDiagnosticPoolEntry, FixedDiagnosticPool},
      DiagnosticPoolBuilder,
    },
  },
  error::{
    context::ErrorContext,
    render::{
      buffer::{
        cell::{
          tag::{builtin::report::REPORT_INFO_LINE_TEXT, context::RenderMode},
          RenderBufferCell,
        },
        RenderBuffer,
      },
      position::RenderPosition,
    },
    FileforgeError, RenderableResult,
  },
  provider::hint::ReadHint,
  stream::{builtin::provider::ProviderStream, ReadableStream, ResizableStream},
};
use fileforge_macros::text;
use fileforge_nintendo::sead::yaz0::{
  readable::{Immutable, Mutable},
  Yaz0Stream,
};
use tokio::fs;

struct AwSoSadError;

#[tokio::main]
async fn main() {
  let pool = DynamicDiagnosticPool::new();

  let sl = include_bytes!("../binaries/SkyWorldHomeStageMap.szs");
  // let sl = include_bytes!("T:\\unsorted-torrents\\Super Mario 3D World\\Super Mario 3D World [ARDP01]\\content\\ObjectData\\ArrangeHexScrollStepA.szs");

  let mut bytes = Vec::from_iter(sl.iter().copied());

  let s = ProviderStream::new(&mut bytes, ReadHint::new());
  let mut r = BinaryReader::new(s, Endianness::BigEndian);

  r.set_diagnostic(DiagnosticKind::Reader, Some(pool.create(DiagnosticBranch::None, Some(sl.len() as u64), "SkyWorldHomeStageMap.szs")));

  let mut val = r.into_with::<Yaz0Stream<_, _>>(Mutable).await.unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool);

  val.skip(0xA8).await.unwrap();
  val.overwrite(3, *b"AAAA").await;

  let mut out: Vec<u8> = Vec::with_capacity(val.len().unwrap() as usize + 0x111);
  out.resize(val.len().unwrap() as usize + 0x111, 0xDE);

  fs::write("./post.bin.yaz0", &bytes).await.unwrap();

  let res = yaz0::inflate::Yaz0Archive::new(Cursor::new(&sl[..])).unwrap().decompress().unwrap();

  fs::write("./pre.bin", res).await.unwrap();

  if let Err(e) = yaz0::inflate::Yaz0Archive::new(Cursor::new(&bytes)).unwrap().decompress_into(&mut out[..]) {
    println!("Failed to write post: {e:?}");
  }

  fs::write("./post.bin", out).await.unwrap();
}
