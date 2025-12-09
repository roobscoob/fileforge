use crate::binary_reader::endianness::Endianness::{self, *};

use crate::{
  binary_reader::{
    error::{common::Read, primitive_name_annotation::PrimitiveName, GetPrimitiveError},
    readable::Readable,
    BinaryReader, PrimitiveReader,
  },
  error::ext::annotations::annotated::Annotated,
  stream::ReadableStream,
};

use super::Primitive;

macro_rules! numeric {
  ($size: expr, $type: ident) => {
    impl Primitive<$size> for $type {
      fn read(data: &[u8; $size], endianness: Endianness) -> Self {
        match endianness {
          LittleEndian => Self::from_le_bytes(*data),
          BigEndian => Self::from_be_bytes(*data),
        }
      }

      fn write(&self, data: &mut [u8; $size], endianness: Endianness) {
        match endianness {
          LittleEndian => *data = self.to_le_bytes(),
          BigEndian => *data = self.to_be_bytes(),
        }
      }
    }

    impl<'pool, S: ReadableStream<Type = u8>> Readable<'pool, S> for $type {
      type Argument = ();
      type Error = Annotated<PrimitiveName<Read>, GetPrimitiveError<'pool, S::ReadError>>;

      const SIZE: Option<u64> = Some($size);

      async fn read(reader: &mut BinaryReader<'pool, S>, _: Self::Argument) -> Result<Self, Self::Error> {
        reader.get().await
      }

      fn measure(&self) -> Option<u64> {
        Some($size)
      }
    }
  };
}

numeric!(1, u8);
numeric!(2, u16);
numeric!(4, u32);
numeric!(8, u64);
numeric!(16, u128);
numeric!(1, i8);
numeric!(2, i16);
numeric!(4, i32);
numeric!(8, i64);
numeric!(16, i128);
numeric!(4, f32);
numeric!(8, f64);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct u24(u32);
numeric!(3, u24);

impl u24 {
  pub fn from_le_bytes(bytes: [u8; 3]) -> Self {
    let value = u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16);
    u24(value)
  }

  pub fn from_be_bytes(bytes: [u8; 3]) -> Self {
    let value = (u32::from(bytes[0]) << 16) | (u32::from(bytes[1]) << 8) | u32::from(bytes[2]);
    u24(value)
  }

  pub fn to_le_bytes(self) -> [u8; 3] {
    [(self.0 & 0xFF) as u8, ((self.0 >> 8) & 0xFF) as u8, ((self.0 >> 16) & 0xFF) as u8]
  }

  pub fn to_be_bytes(self) -> [u8; 3] {
    [((self.0 >> 16) & 0xFF) as u8, ((self.0 >> 8) & 0xFF) as u8, (self.0 & 0xFF) as u8]
  }
}

impl Into<u32> for u24 {
  fn into(self) -> u32 {
    self.0
  }
}
