use crate::{diagnostic::{node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference}, pool::DiagnosticPool}, provider::r#trait::Provider};

use self::{endianness::Endianness, error::{out_of_bounds::ReadOutOfBoundsError, ParseError, ParsePrimitiveError}, r#trait::{none_sized_argument::NoneSizedArgument, primitive::Primitive, readable::{DynamicSizeReadable, FixedSizeReadable}}};

pub mod error;
pub mod r#trait;
pub mod endianness;
pub mod primitive;

pub struct Reader<'pool_lifetime, 'provider_lifetime, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  pub(self) provider: &'provider_lifetime mut P,
  pub(self) endianness: Endianness,
  pub(self) offset: u64,
  pub(self) diagnostic_reference: DiagnosticReference<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE>,
}

impl<'pool_lifetime, 'provider_lifetime, const DIAGNOSTIC_NODE_NAME_SIZE: usize, UnderlyingProvider: Provider> 
  Reader<'pool_lifetime, 'provider_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider>
{
  pub fn remaining(&self) -> u64 {
    self.provider.len() - self.offset
  }

  pub fn diagnostic_reference(&self) -> DiagnosticReference<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference
  }

  pub fn root(provider: &'provider_lifetime mut UnderlyingProvider, endianness: Endianness, pool: &'pool_lifetime DiagnosticPool<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE>, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Reader<'pool_lifetime, 'provider_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider> {
    let reference = pool.try_create(DiagnosticBranch::None, provider.len(), name);
    
    Reader { 
      provider,
      endianness,
      offset: 0,
      diagnostic_reference: reference,
    }
  }
}

impl<'pool_lifetime, 'provider_lifetime, const DIAGNOSTIC_NODE_NAME_SIZE: usize, UnderlyingProvider: Provider>
  Reader<'pool_lifetime, 'provider_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider>
{
  pub fn get<const PRIMITIVE_SIZE: usize, P: Primitive<PRIMITIVE_SIZE>>(&mut self, name: &'static str) -> Result<P, ParsePrimitiveError<'pool_lifetime, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.provider.with_read(self.offset, |data: &[u8; PRIMITIVE_SIZE]| {
      match self.endianness {
        Endianness::Big => P::read_be(data),
        Endianness::Little => P::read_le(data),
      }
    })
      .map_err(|e| ParsePrimitiveError::from_read_error(e, self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name))))?
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name))))?)
  }

  pub fn read<const TYPE_SIZE: usize, T: FixedSizeReadable<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, TYPE_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<T, ParseError<'pool_lifetime, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>>
    where T::Argument: NoneSizedArgument
  {
    let dr = self.diagnostic_reference();

    let mut slice = self.provider.slice::<TYPE_SIZE>(self.offset)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: &mut slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, TYPE_SIZE as u64, name)
    };

    T::read(&mut child, T::Argument::from_none())
  }

  pub fn read_with<const TYPE_SIZE: usize, T: FixedSizeReadable<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE, TYPE_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>, argument: T::Argument) -> Result<T, ParseError<'pool_lifetime, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let dr = self.diagnostic_reference();

    let mut slice = self.provider.slice::<TYPE_SIZE>(self.offset)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: &mut slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, TYPE_SIZE as u64, name)
    };

    T::read(&mut child, argument)
  }

  pub fn read_dyn<T: DynamicSizeReadable<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<T, ParseError<'pool_lifetime, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>>
    where T::Argument: NoneSizedArgument
  {
    let dr = self.diagnostic_reference();

    let size = T::get_size().unwrap_or(self.remaining());
    let mut slice = self.provider.slice_dyn(self.offset, size)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: &mut slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, size, name)
    };

    T::read(&mut child, T::Argument::from_none())
  }

  pub fn read_dyn_with<T: DynamicSizeReadable<'pool_lifetime, DIAGNOSTIC_NODE_NAME_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>, argument: T::Argument) -> Result<T, ParseError<'pool_lifetime, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let dr = self.diagnostic_reference();

    let size = T::get_size().unwrap_or(self.remaining());
    let mut slice = self.provider.slice_dyn(self.offset, size)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: &mut slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, size, name)
    };

    T::read(&mut child, argument)
  }
}