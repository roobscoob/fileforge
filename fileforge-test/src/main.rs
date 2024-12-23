use fileforge_byml::unmanaged::{
  node::{dictionary::BymlDictionaryNodeReader, string::BymlStringNodeReader},
  BymlReader,
};
use fileforge_lib::{
  diagnostic::{
    node::{branch::DiagnosticBranch, name::DiagnosticNodeName},
    pool::{entry::DiagnosticPoolEntry, DiagnosticPool},
  },
  error::{Error, ErrorResultExt},
  provider::{builtin::rust_slice::RustSliceBinaryProvider, r#trait::Provider},
};
use fileforge_std::providers::{file::FileProvider, log::LogProvider};

fn main() {
  let mut provider = FileProvider::open("./fileforge-test/binaries/real.byml").unwrap();
  let mut pool_buffer: [DiagnosticPoolEntry<32>; 32] = Default::default();
  let mut pool = DiagnosticPool::new(&mut pool_buffer);
  let pool_ref = &mut pool;

  let mut log_provider = LogProvider::over(provider);

  let byml_view = BymlReader::over(&mut log_provider, &pool_ref);

  // println!(
  //   "v{}, {:?}",
  //   byml_view.version().unwrap_displayable(),
  //   byml_view.endianness().unwrap_displayable()
  // );

  // if let Some(mut st) = byml_view.string_table().unwrap_displayable() {
  //   println!("StringTable:");

  //   for i in 0..st.length().unwrap_displayable() {
  //     st.try_get(i, |str| {
  //       println!("  {i}: {str:?}");
  //     })
  //     .unwrap_displayable()
  //   }
  // }

  // if let Some(mut kt) = byml_view.key_table().unwrap_displayable() {
  //   println!("KeyTable:");

  //   for i in 0..kt.length().unwrap_displayable() {
  //     kt.try_get(i, |str| {
  //       println!("  {i}: {str:?}");
  //     })
  //     .unwrap_displayable()
  //   }
  // }

  let root = byml_view.root().unwrap_displayable().unwrap();

  let mut dict = root.downcast::<BymlDictionaryNodeReader<32, _>>().unwrap();

  println!(
    "Root (as Dict) len: {}",
    dict.length().map_err(|e| e.into_display()).unwrap()
  );

  let node = dict
    .get(c"ELink2/elink2.Product.belnk")
    .map_err(|e| e.into_display())
    .unwrap();

  match node {
    None => println!("Not Found"),
    Some(_) => println!("Found!"),
  }

  for (i, entry) in dict.into_iter().enumerate() {
    match entry {
      Err(_) => println!("{}: Err", i),

      Ok(v) => {
        let name = v
          .with_node_name(|name| String::from_utf8(Vec::from(name.to_bytes())))
          .unwrap_or_else(|_| panic!("Womp"))
          .unwrap();

        let value = v
          .value()
          .downcast::<BymlStringNodeReader<32, _>>()
          .unwrap_or_else(|_| panic!("Womp"));

        let value = value.with_content(|name| String::from_utf8(Vec::from(name.to_bytes())));

        println!(
          "{} ({}): {:?}",
          i,
          name,
          value.map_err(|e| e.into_display())
        )
      }
    }
  }
}
