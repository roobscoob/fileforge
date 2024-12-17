use alloc::string::String;
use dictionary::Dictionary;
use hash_array::HashArrayRemapped;

use self::{
  binary_data::BinaryData, hash_array::HashArray, heterogeneous_array::HeterogeneousArray,
  homogeneous_array::HomogeneousArray,
};

pub mod binary_data;
pub mod container;
pub mod dictionary;
pub mod hash_array;
pub mod heterogeneous_array;
pub mod homogeneous_array;

pub enum BymlNode {
  // hash array
  HashArray4(HashArray<1>),
  HashArray8(HashArray<2>),
  HashArray12(HashArray<3>),
  HashArray16(HashArray<4>),
  HashArray20(HashArray<5>),
  HashArray24(HashArray<6>),
  HashArray28(HashArray<7>),
  HashArray32(HashArray<8>),
  HashArray36(HashArray<9>),
  HashArray40(HashArray<10>),
  HashArray44(HashArray<11>),
  HashArray48(HashArray<12>),
  HashArray52(HashArray<13>),
  HashArray56(HashArray<14>),
  HashArray60(HashArray<15>),
  HashArrayRemapped4(HashArrayRemapped<1>),
  HashArrayRemapped8(HashArrayRemapped<2>),
  HashArrayRemapped12(HashArrayRemapped<3>),
  HashArrayRemapped16(HashArrayRemapped<4>),
  HashArrayRemapped20(HashArrayRemapped<5>),
  HashArrayRemapped24(HashArrayRemapped<6>),
  HashArrayRemapped28(HashArrayRemapped<7>),
  HashArrayRemapped32(HashArrayRemapped<8>),
  HashArrayRemapped36(HashArrayRemapped<9>),
  HashArrayRemapped40(HashArrayRemapped<10>),
  HashArrayRemapped44(HashArrayRemapped<11>),
  HashArrayRemapped48(HashArrayRemapped<12>),
  HashArrayRemapped52(HashArrayRemapped<13>),
  HashArrayRemapped56(HashArrayRemapped<14>),
  HashArrayRemapped60(HashArrayRemapped<15>),

  // dictionary
  Dictionary(Dictionary),

  // arrays
  HomogeneousArray(HomogeneousArray),
  HeterogeneousArray(HeterogeneousArray),

  // binary data
  BinaryData(BinaryData),

  // primitives
  String(String),
  Boolean(bool),
  S32(i32),
  F32(f32),
  U32(u32),
  S64(i64),
  U64(u64),
  F64(f64),
  Null,
}
