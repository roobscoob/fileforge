use crate::reader::endianness::Endianness::{self, *};

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
