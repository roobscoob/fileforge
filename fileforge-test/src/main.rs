use std::fs;

use fileforge::{
  binary_reader::{endianness::Endianness, BinaryReader},
  diagnostic::pool::dynamic::DynamicDiagnosticPool,
  error::{render::buffer::cell::tag::context::RenderMode, RenderableResult},
  provider::hint::ReadHint,
  stream::extensions::readable::{byte::BinaryReadableStreamExt, ReadableStreamExt},
};
use fileforge_nintendo::{
  byml::{header::BymlHeaderConfig, Byml},
  sead::yaz0::{readable::Immutable, Yaz0Stream},
};
use fileforge_std::encodings::ascii::{codepages::iso_8859_1::Iso8859_1, Ascii};

#[tokio::main]
async fn main() {
  let pool = DynamicDiagnosticPool::new();

  let r = Vec::from_iter(*include_bytes!("../binaries/Bed.byml.yaz0"));
  let r = BinaryReader::new_from_provider(r, Endianness::BigEndian, ReadHint {});
  let r = BinaryReader::new(
    r.into_with::<Yaz0Stream<_, _>>(Immutable).await.unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool),
    Endianness::BigEndian,
  );

  let first_string = r
    .into_with::<Byml<_>>(BymlHeaderConfig::build().without_binary_data_table().build())
    .await
    .unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool)
    .into_key_table()
    .await
    .unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool)
    .expect("Has a literal table")
    .into_string_table()
    .map_err(|_| {})
    .expect("Is a string table")
    .into_string(8)
    .await
    .unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool)
    .decode::<Ascii<Iso8859_1>>()
    .collect::<String>()
    .await
    .unwrap_renderable::<32>(RenderMode::TerminalAnsi, &pool);

  println!("{first_string:?}");
}
