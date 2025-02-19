use fileforge_lib::{
  diagnostic::{node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference}, pool::{fixed::{entry::DiagnosticPoolEntry, FixedDiagnosticPool}, DiagnosticPool}, value::DiagnosticValue}, error::{render::{buffer::{cell::{tag::context::RenderMode, RenderBufferCell}, RenderBuffer}, position::RenderPosition}, FileforgeError, RenderableError, RenderableResult}, provider::{builtins::rust::slice::RustSliceProvider, hint::ReadHint}, reader::{diagnostic_store::DiagnosticKind, endianness::Endianness, error::seek_out_of_bounds::{SeekOffset, SeekOutOfBounds}, PrimitiveReader, Reader}, stream::builtin::provider::ProviderStream
};

#[tokio::main]
async fn main() {
  let mut entries: [DiagnosticPoolEntry<32>; 100] = core::array::from_fn(|_| DiagnosticPoolEntry::default());
  let pool = FixedDiagnosticPool::new(&mut entries);

  let sl: [u8; 4] = [0x89, 0xAB, 0xCD, 0xEF];
  let p = RustSliceProvider::from(&sl);
  let s = ProviderStream::new(p, ReadHint::new());
  let mut r = Reader::new(s, Endianness::BigEndian);

  r.set_diagnostic(DiagnosticKind::Reader, Some(pool.create(DiagnosticBranch::None, Some(3), DiagnosticNodeName::from("sl"))));

  let val = r.get::<u32>().await.unwrap_renderable();

  assert_eq!(val, 0x89ABCDEF)
}
