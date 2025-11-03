use std::{io::Cursor, time::Instant};

use fileforge_lib::{
  binary_reader::{diagnostic_store::DiagnosticKind, endianness::Endianness, BinaryReader},
  diagnostic::{
    node::branch::DiagnosticBranch,
    pool::{
      fixed::{entry::FixedDiagnosticPoolEntry, FixedDiagnosticPool},
      DiagnosticPoolBuilder,
    },
  },
  error::{render::buffer::cell::tag::context::RenderMode, RenderableResult},
  provider::{builtins::rust::slice::RustSliceProvider, hint::ReadHint},
  stream::{builtin::provider::ProviderStream, ReadableStream, ResizableStream},
};
use fileforge_nintendo::sead::yaz0::{
  readable::{Immutable, Mutable},
  Yaz0Stream,
};
use tokio::fs;

#[tokio::main]
async fn main() {
  let mut entries: [FixedDiagnosticPoolEntry<32>; 100] = core::array::from_fn(|_| FixedDiagnosticPoolEntry::default());
  let pool = FixedDiagnosticPool::new(&mut entries);

  let sl = include_bytes!("../binaries/SkyWorldHomeStageMap.szs");
  // let sl = include_bytes!("T:\\unsorted-torrents\\Super Mario 3D World\\Super Mario 3D World [ARDP01]\\content\\ObjectData\\ArrangeHexScrollStepA.szs");

  let bytes = Vec::from_iter(sl.iter().copied());

  let p = bytes;
  let mut s = ProviderStream::new(p, ReadHint::new());
  let mut r = BinaryReader::new(&mut s, Endianness::BigEndian);

  r.set_diagnostic(DiagnosticKind::Reader, Some(pool.create(DiagnosticBranch::None, Some(sl.len() as u64), "SkyWorldHomeStageMap.szs")));

  let mut val = r.into_with::<Yaz0Stream<_, _>>(Mutable).await.unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool);

  val.skip(0xA8).await.unwrap();
  val.overwrite(3, *b"Soy").await.unwrap();

  let v = s.into_provider();

  fs::write("./post.bin.yaz0", &v).await.unwrap();

  let res = yaz0::inflate::Yaz0Archive::new(Cursor::new(&sl[..])).unwrap().decompress().unwrap();

  fs::write("./pre.bin", res).await.unwrap();

  let res = yaz0::inflate::Yaz0Archive::new(Cursor::new(&v)).unwrap().decompress().unwrap();

  fs::write("./post.bin", res).await.unwrap();
}
