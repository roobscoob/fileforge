use fileforge::{
  binary_reader::{endianness::Endianness, error::get_primitive::GetPrimitiveError, readable::Readable, PrimitiveReader},
  error::FileforgeError,
  stream::{error::user_read::UserReadError, ReadableStream},
};

use fileforge_std::{
  byte_order_mark::{error::error::ByteOrderMarkError, ByteOrderMark},
  magic::{error::error::MagicError, Magic},
};

use crate::sead::sarc::structures::header::SarcHeader;

pub const SARC_MAGIC: Magic<4> = Magic::from_byte_ref(b"SARC");
pub const SARC_BOM: ByteOrderMark = ByteOrderMark::from_byte_ref(Endianness::BigEndian, &[0xFE, 0xFF]);

impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for SarcHeader {
  type Error = SarcHeaderReadError<'pool, S::ReadError>;
  type Argument = ();

  async fn read(reader: &mut fileforge::binary_reader::BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
    reader.read_with::<Magic<4>>(SARC_MAGIC).await.map_err(|e| SarcHeaderReadError::Magic(e))?;

    let _header_length: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::HeaderLength(e))?;

    let endianness = reader.read_with::<ByteOrderMark>(SARC_BOM).await.map_err(|e| SarcHeaderReadError::BOM(e))?.endianness();

    let mut reader = reader.borrow_fork();

    reader.set_endianness(endianness);

    let size = reader.get().await.map_err(|e| SarcHeaderReadError::Size(e))?;
    let data_section_offset = reader.get().await.map_err(|e| SarcHeaderReadError::DataSectionOffset(e))?;
    let version: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::Version(e))?;
    let version = ((version >> 8) as u8, (version & 0xFF) as u8);

    let _unused: u16 = reader.get().await.map_err(|e| SarcHeaderReadError::Unused(e))?;

    Ok(SarcHeader {
      endianness,
      size,
      version,
      data_section_offset,
    })
  }
}

pub enum SarcHeaderReadError<'pool, U: UserReadError> {
  Magic(MagicError<'pool, 4, U>),
  BOM(ByteOrderMarkError<'pool, U>),
  Size(GetPrimitiveError<'pool, U>),
  DataSectionOffset(GetPrimitiveError<'pool, U>),
  Version(GetPrimitiveError<'pool, U>),
  Unused(GetPrimitiveError<'pool, U>),
  HeaderLength(GetPrimitiveError<'pool, U>),
}

impl<'pool, U: UserReadError> FileforgeError for SarcHeaderReadError<'pool, U> {
  fn render_into_report<'pool_ref, const ITEM_NAME_SIZE: usize, P: fileforge::diagnostic::pool::DiagnosticPoolProvider>(
    &self,
    _provider: &'pool_ref P,
    _callback: impl for<'tag, 'b, 'p2> FnMut(fileforge::error::report::Report<'tag, 'b, 'p2, 'pool_ref, ITEM_NAME_SIZE, P>) -> (),
  ) {
    unimplemented!()
  }
}
