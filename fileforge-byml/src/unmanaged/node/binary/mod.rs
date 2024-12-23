// use std::ffi::CStr;

// use crate::unmanaged::BymlReader;
// use error::{stat_error::StatError, string_node_get_content::StringNodeGetContent};
// use fileforge_lib::{
//   diagnostic::node::name::DiagnosticNodeName,
//   provider::{
//     error::never::Never,
//     r#trait::Provider,
//     slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
//   },
//   reader::error::{
//     parse::ParseError, parse_primitive::ParsePrimitiveError,
//     underlying_provider_stat::UnderlyingProviderStatError,
//   },
// };

// use super::r#trait::BymlNodeReader;

// pub struct BymlBinaryNodeReader<
//   'byml,
//   'byml_provider,
//   'pool,
//   const DIAGNOSTIC_NODE_NAME_SIZE: usize,
//   BP: Provider,
// > {
//   has_alignment: bool,
//   reader: fileforge_lib::reader::Reader<
//     'pool,
//     DIAGNOSTIC_NODE_NAME_SIZE,
//     <BP as Provider>::DynReturnedProviderType<'byml_provider>,
//   >,
//   content: <BP as Provider>::DynReturnedProviderType<'byml_provider>,
//   byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
// }

// impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
//   BymlBinaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
// {
//   pub fn alignment(
//     &self,
//   ) -> Option<
//     Result<
//       u32,
//       ParsePrimitiveError<
//         'pool,
//         <BP as Provider>::ReadError,
//         <BP as Provider>::StatError,
//         DIAGNOSTIC_NODE_NAME_SIZE,
//       >,
//     >,
//   > {
//     if self.has_alignment {
//       Some(self.reader.get_at::<4, u32>("Alignment", 4))
//     } else {
//       None
//     }
//   }
// }

// impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider>
//   BymlNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
//   for BymlBinaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
// {
//   type ReadError =
//     ParsePrimitiveError<'pool, BP::ReadError, BP::StatError, DIAGNOSTIC_NODE_NAME_SIZE>;

//   fn requires_dereferencing(type_id: u8) -> bool { true }
//   fn type_id_supported(type_id: u8) -> bool { type_id == 0xA1 || type_id == 0xA2 }

//   fn from_value(
//     type_id: u8,
//     value: u32,
//     byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
//   ) -> Self {
//     panic!("Cannot create from value");
//   }

//   fn from_reader(
//     type_id: u8,
//     reader: fileforge_lib::reader::Reader<
//       'pool,
//       DIAGNOSTIC_NODE_NAME_SIZE,
//       <BP as Provider>::DynReturnedProviderType<'byml_provider>,
//     >,
//     byml: &'byml BymlReader<'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>,
//   ) -> Result<Self, Self::ReadError> {
//     Ok(Self {
//       has_alignment: type_id == 0xA2,
//       reader,
//       byml,
//     })
//   }
// }

// impl<'byml, 'byml_provider, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, BP: Provider> Provider
//   for BymlBinaryNodeReader<'byml, 'byml_provider, 'pool, DIAGNOSTIC_NODE_NAME_SIZE, BP>
// {
//   type ReadError = BP::ReadError;
//   type StatError = ParsePrimitiveError<
//     'pool,
//     <BP as Provider>::ReadError,
//     <BP as Provider>::StatError,
//     DIAGNOSTIC_NODE_NAME_SIZE,
//   >;
//   type ReturnedProviderType<'underlying, const SIZE: usize>
//     = FixedSliceProvider<'underlying, SIZE, Self>
//   where
//     Self: 'underlying;
//   type DynReturnedProviderType<'underlying>
//     = DynamicSliceProvider<'underlying, Self>
//   where
//     Self: 'underlying;

//   fn slice<const SIZE: usize>(
//     &self,
//     offset: u64,
//   ) -> Result<
//     Self::ReturnedProviderType<'_, SIZE>,
//     fileforge_lib::provider::error::slice_error::SliceError<Self::StatError>,
//   > {
//     FixedSliceProvider::over(&self, offset)
//   }

//   fn slice_dyn(
//     &self,
//     offset: u64,
//     size: u64,
//   ) -> Result<
//     Self::DynReturnedProviderType<'_>,
//     fileforge_lib::provider::error::slice_error::SliceError<Self::StatError>,
//   > {
//     DynamicSliceProvider::over(self, offset, size)
//   }

//   fn len(&self) -> Result<u64, Self::StatError> {
//     self.reader.get_at::<4, u32>("Length", 0).map(|v| v as u64)
//   }
// }
