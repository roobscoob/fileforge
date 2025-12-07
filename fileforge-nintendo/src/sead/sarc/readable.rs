use core::convert::Infallible;

use fileforge::{
  binary_reader::{readable::IntoReadable, BinaryReader},
  stream::ReadableStream,
};

use crate::sead::sarc::{header::SarcHeader, Sarc};

// impl<'pool, S: ReadableStream<Type = u8>> IntoReadable<'pool, S> for Sarc {
//   type Argument = ();
//   type Error = Infallible;

//   async fn read(mut reader: BinaryReader<'pool, S>, argument: Self::Argument) -> Result<Self, Self::Error> {
//     let header: SarcHeader = reader.read().await?;
//   }
// }
