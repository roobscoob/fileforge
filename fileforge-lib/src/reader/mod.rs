use error::underlying_provider_read::UnderlyingProviderReadError;

use crate::{diagnostic::{node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference}, pool::DiagnosticPool}, error::render::{buffer::cell::tag::CellTag, builtin::text::Text, r#trait::renderable::Renderable}, provider::{error::{read_error::ReadError, ProviderError}, out_of_bounds::SliceOutOfBoundsError, slice::{dynamic::DynamicSliceProvider, fixed::{FixedMutSliceProvider, FixedSliceProvider}}, r#trait::Provider}};

use self::{endianness::Endianness, error::{out_of_bounds::ReadOutOfBoundsError, parse::ParseError, parse_primitive::ParsePrimitiveError, expect_primitive::ExpectPrimitiveError, expect_primitive::ExpectationFailedError}, r#trait::{none_sized_argument::NoneSizedArgument, primitive::Primitive, readable::{DynamicSizeReadable, FixedSizeReadable}}};

pub mod error;
pub mod r#trait;
pub mod endianness;
pub mod primitive;

pub enum SeekFrom {
  Start(u64),
  End(i64),
  Current(i64),
}

pub struct Reader<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  pub(self) provider: P,
  pub(self) endianness: Endianness,
  pub(self) offset: u64,
  pub(self) diagnostic_reference: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, UnderlyingProvider: Provider> 
  Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider>
{
  pub fn remaining(&self) -> u64 {
    self.provider.len() - self.offset
  }

  pub fn len(&self) -> u64 {
    self.provider.len()
  }

  pub fn diagnostic_reference(&self) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference
  }

  pub fn root(provider: UnderlyingProvider, endianness: Endianness, pool: &'pool DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider> {
    let reference = pool.try_create(DiagnosticBranch::None, provider.len(), name);
    
    Reader { 
      provider,
      endianness,
      offset: 0,
      diagnostic_reference: reference,
    }
  }

  pub fn at(provider: UnderlyingProvider, endianness: Endianness, at: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) -> Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider> { 
    Reader { 
      provider,
      endianness,
      offset: 0,
      diagnostic_reference: at,
    }
  }

  pub fn offset(&self) -> u64 {
    self.offset
  }

  pub fn set_endianness(&mut self, endianness: Endianness) {
    self.endianness = endianness;
  }

  pub fn slice<'s, const SIZE: usize>(&'s self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, FixedSliceProvider<'s, SIZE, <UnderlyingProvider as Provider>::ReturnedProviderType>>, SliceOutOfBoundsError> {
    let dr = self.diagnostic_reference();
    let slice = self.provider.slice::<SIZE>(self.offset)?;

    Ok(Reader {
      diagnostic_reference: dr.create_physical_child(self.offset, SIZE as u64, name),
      provider: slice,
      endianness: self.endianness,
      offset: 0,
    })
  }

  pub fn slice_dyn<'s>(&'s mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>, size: Option<u64>) -> Result<Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, DynamicSliceProvider<'s, <UnderlyingProvider as Provider>::DynReturnedProviderType>>, SliceOutOfBoundsError> {
    let dr = self.diagnostic_reference();
    let len = size.unwrap_or(self.remaining());
    let slice = self.provider.slice_dyn(self.offset, len)?;

    Ok(Reader {
      diagnostic_reference: dr.create_physical_child(self.offset, len, name),
      provider: slice,
      endianness: self.endianness,
      offset: 0,
    })
  }

  pub fn get<const PRIMITIVE_SIZE: usize, P: Primitive<PRIMITIVE_SIZE>>(&mut self, name: &'static str) -> Result<P, ParsePrimitiveError<'pool, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let v = self.provider.with_read(self.offset, |data: &[u8; PRIMITIVE_SIZE]| {
      match self.endianness {
        Endianness::Big => P::read_be(data),
        Endianness::Little => P::read_le(data),
      }
    })
      .map_err(|e| ParsePrimitiveError::from_read_error(e, self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name))))?
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name))))?;

    self.offset += PRIMITIVE_SIZE as u64;

    Ok(v)
  }

  pub fn expect<const PRIMITIVE_SIZE: usize, P: Primitive<PRIMITIVE_SIZE>>(&mut self, name: &'static str, expect_fn: impl FnOnce(&P) -> bool, error_fn: for<'f, 'g> fn(&'f P, &'g mut (dyn for<'tag, 'text_data, 'renderable> FnMut(Text<'text_data, 'tag>, &'tag (dyn CellTag + 'tag), Option<&'renderable (dyn Renderable<'tag> + 'renderable)>) + 'g))) -> Result<P, ExpectPrimitiveError<'pool, UnderlyingProvider::ReadError, P, PRIMITIVE_SIZE, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let v = self.provider.with_read(self.offset, |data: &[u8; PRIMITIVE_SIZE]| {
      match self.endianness {
        Endianness::Big => P::read_be(data),
        Endianness::Little => P::read_le(data),
      }
    })
      .map_err(|e| ParsePrimitiveError::from_read_error(e, self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name))))?
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name))))?;

    if expect_fn(&v) {
      self.offset += PRIMITIVE_SIZE as u64;
  
      return Ok(v)
    };

    let dr = self.diagnostic_reference().create_physical_child(self.offset, PRIMITIVE_SIZE as u64, DiagnosticNodeName::from(name));

    Err(ExpectPrimitiveError::ExpectationFailed(ExpectationFailedError(v, dr, error_fn)))
  }

  pub fn read<const TYPE_SIZE: usize, T: FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, TYPE_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<T, ParseError<'pool, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>>
    where T::Argument: NoneSizedArgument
  {
    let dr = self.diagnostic_reference();

    let slice = self.provider.slice::<TYPE_SIZE>(self.offset)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, TYPE_SIZE as u64, name)
    };

    self.offset += TYPE_SIZE as u64;

    T::read(&mut child, T::Argument::from_none())
  }

  pub fn read_with<const TYPE_SIZE: usize, T: FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, TYPE_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>, argument: T::Argument) -> Result<T, ParseError<'pool, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let dr = self.diagnostic_reference();

    let slice = self.provider.slice::<TYPE_SIZE>(self.offset)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, TYPE_SIZE as u64, name)
    };

    self.offset += TYPE_SIZE as u64;

    T::read(&mut child, argument)
  }

  pub fn read_dyn<T: DynamicSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>) -> Result<T, ParseError<'pool, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>>
    where T::Argument: NoneSizedArgument
  {
    let dr = self.diagnostic_reference();

    let size = T::get_size(self)?.unwrap_or(self.remaining());
    let mut slice = self.provider.slice_dyn(self.offset, size)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, size, name)
    };

    self.offset += size;

    T::read(&mut child, T::Argument::from_none())
  }

  pub fn read_dyn_with<T: DynamicSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(&mut self, name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>, argument: T::Argument) -> Result<T, ParseError<'pool, T::Error, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let dr = self.diagnostic_reference();

    let size = T::get_size(self)?.unwrap_or(self.remaining());
    let mut slice = self.provider.slice_dyn(self.offset, size)
      .map_err(|e| ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, dr))?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: 0,
      diagnostic_reference: dr.create_physical_child(self.offset, size, name)
    };

    self.offset += size;

    T::read(&mut child, argument)
  }

  pub fn seek(&mut self, position: SeekFrom) -> Result<(), ()> {
    let (new_value, overflowed) = match position {
      SeekFrom::Current(bytes) => self.offset.overflowing_add_signed(bytes),
      SeekFrom::Start(bytes) => (bytes, false),
      SeekFrom::End(bytes) => self.provider.len().overflowing_add_signed(bytes),
    };

    if overflowed || new_value >= self.provider.len() {
      Err(())
    } else {
      self.offset = new_value;

      Ok(())
    }
  }

  pub fn with_dyn_bytes<T>(&self, length: Option<u64>, name: &str, callback: impl FnOnce(&[u8]) -> T) -> Result<T, ParsePrimitiveError<'pool, UnderlyingProvider::ReadError, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(self.provider.with_read_dyn(self.offset, length.unwrap_or(self.remaining()), callback)
      .map_err(|re| ParsePrimitiveError::UnderlyingProviderReadError(UnderlyingProviderReadError(re.0, self.diagnostic_reference.create_physical_child(self.offset, length.unwrap_or(self.remaining()), DiagnosticNodeName::from(name)))))?
      .map_err(|e| ParsePrimitiveError::OutOfBounds(ReadOutOfBoundsError::from_slice_out_of_bounds_error(e, self.diagnostic_reference.create_physical_child(self.offset, length.unwrap_or(self.remaining()), DiagnosticNodeName::from(name)))))?)
  }
}