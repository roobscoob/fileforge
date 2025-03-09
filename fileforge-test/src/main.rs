use std::{io::Cursor, time::Instant};

use fileforge_lib::{
  diagnostic::{node::branch::DiagnosticBranch, pool::{fixed::{entry::FixedDiagnosticPoolEntry, FixedDiagnosticPool}, DiagnosticPoolBuilder}}, error::{render::buffer::cell::tag::context::RenderMode, RenderableResult}, provider::{builtins::rust::slice::RustSliceProvider, hint::ReadHint}, reader::{diagnostic_store::DiagnosticKind, endianness::Endianness, Reader}, stream::{builtin::provider::ProviderStream, ReadableStream}
};
use fileforge_nintendo::yaz0::Yaz0Stream;
use tokio::fs;

#[tokio::main]
async fn main() {
  // let mut entries: [FixedDiagnosticPoolEntry<32>; 100] = core::array::from_fn(|_| FixedDiagnosticPoolEntry::default());
  // let pool = FixedDiagnosticPool::new(&mut entries);

  // // let sl = include_bytes!("../binaries/SkyWorldHomeStageMap.szs");
  // let sl = include_bytes!("T:\\unsorted-torrents\\Super Mario 3D World\\Super Mario 3D World [ARDP01]\\content\\ObjectData\\ArrangeHexScrollStepA.szs");

  // let p = RustSliceProvider::from(sl);
  // let s = ProviderStream::new(p, ReadHint::new());
  // let mut r = Reader::new(s, Endianness::BigEndian);

  // r.set_diagnostic(DiagnosticKind::Reader, Some(pool.create(DiagnosticBranch::None, Some(sl.len() as u64), "SkyWorldHomeStageMap.szs")));

  // let mut val = r.read::<Yaz0Stream<_>>().await.unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool);

  // let mut data = vec![];

  // let now = Instant::now();
  // println!("Start");

  // for i in 0.. {
  //   let result = val.read(|input: &[u8; 2]| {
  //     for item in input {
  //       data.push(*item);
  //     }

  //     async {}
  //   }).await;

  //   if let Err(_) = result {
  //     println!("Failed at {i}");
  //     break;
  //   }
  // }

  // println!("Stop: {}", now.elapsed().as_millis());
  
  // fs::write("./bad.bin", data).await.unwrap();
}
