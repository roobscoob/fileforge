use fileforge::{
  binary_reader::{endianness::Endianness, BinaryReader},
  diagnostic::pool::dynamic::DynamicDiagnosticPool,
  error::{render::buffer::cell::tag::context::RenderMode, RenderableResult},
  provider::hint::ReadHint,
  stream::extensions::readable::{byte::BinaryReadableStreamExt, ReadableStreamExt},
};
use fileforge_nintendo::byml::header::{BymlHeader, BymlHeaderConfig};
use fileforge_std::encodings::ascii::{codepages::iso_8859_1::Iso8859_1, Ascii};

#[tokio::main]
async fn main() {
  let pool = DynamicDiagnosticPool::new();

  let r = Vec::from_iter(*include_bytes!("../binaries/real.byml"));
  let r = BinaryReader::new_from_provider(r, Endianness::BigEndian, ReadHint {});

  let val = r.into_stream().decode::<Ascii<Iso8859_1>>().collect::<String>().await.unwrap();

  println!("{val:?}");
}
