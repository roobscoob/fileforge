use fileforge_byml::unmanaged::{node::dictionary::BymlDictionaryNodeReader, BymlReader};
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

  let byml_view = BymlReader::over(&mut provider, &pool_ref)
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

  let mut root = byml_view
    .root()
    .map_err(|e| e.into_display())
    .unwrap()
    .unwrap()
    .downcast::<BymlDictionaryNodeReader<32, RustSliceBinaryProvider>>()
    .unwrap();

  println!(
    "Root (as Dict) len: {}",
    root.length().map_err(|e| e.into_display()).unwrap()
  );

  for (i, entry) in root.iter().enumerate() {
    match entry {
      Err(_) => println!("{}: Err", i),

      Ok(v) => {
        let name = v.with_node_name(|name| String::from_utf8(Vec::from(name.to_bytes())));

        match name {
          Ok(Ok(n)) => println!("{} ({}): {}", i, n, v.r#type),
          Ok(Err(_)) => println!("{}: InvalidUtf8", i),
          Err(e) => println!("{}: NameErr {:?}", i, e.into_display()),
        }
      }
    }
  }

  let node = root
    .get(c"ELink2/elink2.Product.belnk")
    .map_err(|e| e.into_display())
    .unwrap();

  match node {
    None => println!("Not Found"),
    Some(_) => println!("Found!"),
  }
}
