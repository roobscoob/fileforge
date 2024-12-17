use fileforge_byml::unmanaged::BymlReader;
use fileforge_lib::{
  diagnostic::pool::{entry::DiagnosticPoolEntry, DiagnosticPool},
  error::Error,
  provider::builtin::rust_slice::RustSliceBinaryProvider,
};

fn main() {
  let bytes = include_bytes!("../binaries/real.byml");
  let mut provider = RustSliceBinaryProvider::over(bytes);
  let mut pool_buffer: [DiagnosticPoolEntry<32>; 32] = Default::default();
  let mut pool = DiagnosticPool::new(&mut pool_buffer);
  let pool_ref = &mut pool;

  let mut byml_view = BymlReader::over(&mut provider, &pool_ref)
    .map_err(|e| e.into_display())
    .unwrap();
  let version = byml_view.version().map_err(|e| e.into_display()).unwrap();
  let endianness = byml_view
    .endianness()
    .map_err(|e| e.into_display())
    .unwrap();

  println!("version: {}", version);
  println!("endianness: {:?}", endianness);

  let mut st = byml_view
    .string_table()
    .map_err(|e| e.into_display())
    .unwrap();

  println!("StringTable:");
  for i in 0..st.length().map_err(|e| e.into_display()).unwrap() {
    st.try_get(i, |str| {
      println!("  {i}: {str:?}");
    })
    .map_err(|e| e.into_display())
    .unwrap()
  }

  let mut kt = byml_view.key_table().map_err(|e| e.into_display()).unwrap();

  println!("KeyTable:");
  for i in 0..kt.length().map_err(|e| e.into_display()).unwrap() {
    kt.try_get(i, |str| {
      println!("  {i}: {str:?}");
    })
    .map_err(|e| e.into_display())
    .unwrap()
  }

  let root = byml_view.root().map_err(|e| e.into_display()).unwrap();
}
