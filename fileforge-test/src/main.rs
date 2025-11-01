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
  error::{
    render::{
      buffer::{
        cell::{tag::context::RenderMode, RenderBufferCell},
        RenderBuffer,
      },
      grapheme::Grapheme,
      position::RenderPosition,
    },
    FileforgeError, RenderableResult,
  },
  provider::{builtins::rust::slice::RustSliceProvider, hint::ReadHint},
  stream::{builtin::provider::ProviderStream, error::stream_read::StreamReadError, ReadableStream},
};
use fileforge_nintendo::sead::yaz0::Yaz0Stream;
use tokio::fs;

#[tokio::main]
async fn main() {
  let mut entries: [FixedDiagnosticPoolEntry<32>; 100] = core::array::from_fn(|_| FixedDiagnosticPoolEntry::default());
  let pool = FixedDiagnosticPool::new(&mut entries);

  let sl = include_bytes!("../binaries/SkyWorldHomeStageMap.szs");
  // let sl = include_bytes!("T:\\unsorted-torrents\\Super Mario 3D World\\Super Mario 3D World [ARDP01]\\content\\ObjectData\\ArrangeHexScrollStepA.szs");

  let p = RustSliceProvider::from(sl);
  let s = ProviderStream::new(p, ReadHint::new());
  let mut r = BinaryReader::new(s, Endianness::BigEndian);

  r.set_diagnostic(DiagnosticKind::Reader, Some(pool.create(DiagnosticBranch::None, Some(sl.len() as u64), "SkyWorldHomeStageMap.szs")));

  let mut val = r.into::<Yaz0Stream<_>>().await.unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool);

  let mut data = vec![];

  println!("Length = {:?}", val.len());

  let now = Instant::now();

  for i in 0.. {
    let r = val
      .read(|v: &[u8; 0x1000]| {
        data.extend_from_slice(v);
        async {}
      })
      .await;

    if let Err(e) = r {
      println!("{e:?}");
      break;
    }
  }

  println!("Fileforge: {}", now.elapsed().as_millis());

  fs::write("./bad.bin", data).await.unwrap();

  let now = Instant::now();
  let res = yaz0::inflate::Yaz0Archive::new(Cursor::new(&sl[..])).unwrap().decompress().unwrap();
  println!("yaz0 Crate: {}", now.elapsed().as_millis());

  fs::write("./good.bin", res).await.unwrap();
}
