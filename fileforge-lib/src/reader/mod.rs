use core::cell::Cell;

use error::{
  seek::SeekError, underlying_provider_read::UnderlyingProviderReadError,
  underlying_provider_stat::UnderlyingProviderStatError,
};

use crate::{
  diagnostic::{
    node::{branch::DiagnosticBranch, name::DiagnosticNodeName, reference::DiagnosticReference},
    pool::DiagnosticPool,
  },
  error::render::{
    buffer::cell::tag::CellTag, builtin::text::Text, r#trait::renderable::Renderable,
  },
  provider::{
    error::{never::Never, slice_error::SliceError},
    out_of_bounds::SliceOutOfBoundsError,
    r#trait::Provider,
    slice::{dynamic::DynamicSliceProvider, fixed::FixedSliceProvider},
  },
};

use self::{
  endianness::Endianness,
  error::{
    expect_primitive::ExpectPrimitiveError, expect_primitive::ExpectationFailedError,
    out_of_bounds::ReadOutOfBoundsError, parse::ParseError, parse_primitive::ParsePrimitiveError,
  },
  r#trait::{
    none_sized_argument::NoneSizedArgument,
    primitive::Primitive,
    readable::{DynamicSizeReadable, FixedSizeReadable},
  },
};

pub mod endianness;
pub mod error;
pub mod primitive;
pub mod r#trait;

pub enum SeekFrom {
  Start(u64),
  End(i64),
  Current(i64),
}

pub struct Reader<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  pub(self) provider: P,
  pub(self) endianness: Endianness,
  pub(self) offset: Cell<u64>,
  pub(self) diagnostic_reference: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider + Clone> Clone
  for Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>
{
  fn clone(&self) -> Self {
    Reader {
      provider: self.provider.clone(),
      endianness: self.endianness,
      offset: Cell::new(self.offset.get()),
      diagnostic_reference: self.diagnostic_reference,
    }
  }
}

impl<'l, 'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, UnderlyingProvider: Provider>
  Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider>
{
  pub fn remaining(&self) -> Result<u64, UnderlyingProvider::StatError> {
    Ok(self.provider.len()? - self.offset.get())
  }

  pub fn len(&self) -> Result<u64, <UnderlyingProvider as Provider>::StatError> {
    self.provider.len()
  }

  pub fn endianness(&self) -> Endianness { self.endianness }

  pub fn diagnostic_reference(&self) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.diagnostic_reference
  }

  pub fn root(
    provider: UnderlyingProvider,
    endianness: Endianness,
    pool: &'pool DiagnosticPool<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider> {
    let reference = pool.try_create(DiagnosticBranch::None, None, name);

    Reader {
      provider,
      endianness,
      offset: Cell::new(0),
      diagnostic_reference: reference,
    }
  }

  pub fn at(
    provider: UnderlyingProvider,
    endianness: Endianness,
    at: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, UnderlyingProvider> {
    Reader {
      provider,
      endianness,
      offset: Cell::new(0),
      diagnostic_reference: at,
    }
  }

  pub fn offset(&self) -> u64 { self.offset.get() }

  pub fn set_endianness(&mut self, endianness: Endianness) { self.endianness = endianness; }

  pub fn slice<'s, const SIZE: usize>(
    &'s self,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Result<
    Reader<
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      <UnderlyingProvider as Provider>::ReturnedProviderType<'s, SIZE>,
    >,
    SliceError<UnderlyingProvider::StatError>,
  > {
    let dr = self.diagnostic_reference();
    let slice = self.provider.slice::<SIZE>(self.offset())?;

    Ok(Reader {
      diagnostic_reference: dr.create_physical_child(self.offset(), Some(SIZE as u64), name),
      provider: slice,
      endianness: self.endianness,
      offset: Cell::new(0),
    })
  }

  pub fn slice_dyn<'s>(
    &'s self,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
    size: Option<u64>,
  ) -> Result<
    Reader<
      'pool,
      DIAGNOSTIC_NODE_NAME_SIZE,
      <UnderlyingProvider as Provider>::DynReturnedProviderType<'s>,
    >,
    SliceError<UnderlyingProvider::StatError>,
  > {
    let dr = self.diagnostic_reference();
    let slice = self.provider.slice_dyn(self.offset.get(), size)?;

    Ok(Reader {
      diagnostic_reference: dr.create_physical_child(self.offset.get(), size, name),
      provider: slice,
      endianness: self.endianness,
      offset: Cell::new(0),
    })
  }

  pub fn get<const PRIMITIVE_SIZE: usize, P: Primitive<PRIMITIVE_SIZE>>(
    &self,
    name: &'static str,
  ) -> Result<
    P,
    ParsePrimitiveError<
      'pool,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let v = self
      .provider
      .with_read(
        self.offset.get(),
        |data: &[u8; PRIMITIVE_SIZE]| match self.endianness {
          Endianness::Big => P::read_be(data),
          Endianness::Little => P::read_le(data),
        },
      )
      .map_err(|e| {
        ParsePrimitiveError::from_read_error(
          e,
          self.diagnostic_reference().create_physical_child(
            self.offset.get(),
            Some(PRIMITIVE_SIZE as u64),
            DiagnosticNodeName::from(name),
          ),
        )
      })?
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => {
          ParsePrimitiveError::OutOfBounds(ReadOutOfBoundsError::from_slice_out_of_bounds_error(
            oob,
            self.diagnostic_reference().create_physical_child(
              self.offset.get(),
              Some(PRIMITIVE_SIZE as u64),
              DiagnosticNodeName::from(name),
            ),
          ))
        }
        SliceError::StatError(se) => ParsePrimitiveError::UnderlyingProviderStatError(se),
      })?;

    self.offset.set(PRIMITIVE_SIZE as u64 + self.offset());

    Ok(v)
  }

  pub fn get_at<const PRIMITIVE_SIZE: usize, P: Primitive<PRIMITIVE_SIZE>>(
    &self,
    name: &'static str,
    offset: u64,
  ) -> Result<
    P,
    ParsePrimitiveError<
      'pool,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let v = self
      .provider
      .with_read(offset, |data: &[u8; PRIMITIVE_SIZE]| {
        match self.endianness {
          Endianness::Big => P::read_be(data),
          Endianness::Little => P::read_le(data),
        }
      })
      .map_err(|e| {
        ParsePrimitiveError::from_read_error(
          e,
          self.diagnostic_reference().create_physical_child(
            offset,
            Some(PRIMITIVE_SIZE as u64),
            DiagnosticNodeName::from(name),
          ),
        )
      })?
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => {
          ParsePrimitiveError::OutOfBounds(ReadOutOfBoundsError::from_slice_out_of_bounds_error(
            oob,
            self.diagnostic_reference().create_physical_child(
              offset,
              Some(PRIMITIVE_SIZE as u64),
              DiagnosticNodeName::from(name),
            ),
          ))
        }
        SliceError::StatError(se) => ParsePrimitiveError::UnderlyingProviderStatError(se),
      })?;

    self.offset.set(PRIMITIVE_SIZE as u64 + self.offset());

    Ok(v)
  }

  pub fn expect<const PRIMITIVE_SIZE: usize, P: Primitive<PRIMITIVE_SIZE>>(
    &self,
    name: &'static str,
    expect_fn: impl FnOnce(&P) -> bool,
    error_fn: for<'f, 'g> fn(
      &'f P,
      &'g mut (dyn for<'tag, 'text_data, 'renderable> FnMut(
        Text<'text_data, 'tag>,
        &'tag (dyn CellTag + 'tag),
        Option<&'renderable (dyn Renderable<'tag> + 'renderable)>,
      ) + 'g),
    ),
  ) -> Result<
    P,
    ExpectPrimitiveError<
      'pool,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      P,
      PRIMITIVE_SIZE,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let v = self
      .provider
      .with_read(self.offset(), |data: &[u8; PRIMITIVE_SIZE]| {
        match self.endianness {
          Endianness::Big => P::read_be(data),
          Endianness::Little => P::read_le(data),
        }
      })
      .map_err(|e| {
        ParsePrimitiveError::from_read_error(
          e,
          self.diagnostic_reference().create_physical_child(
            self.offset(),
            Some(PRIMITIVE_SIZE as u64),
            DiagnosticNodeName::from(name),
          ),
        )
      })?
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => {
          ExpectPrimitiveError::OutOfBounds(ReadOutOfBoundsError::from_slice_out_of_bounds_error(
            oob,
            self.diagnostic_reference().create_physical_child(
              self.offset(),
              Some(PRIMITIVE_SIZE as u64),
              DiagnosticNodeName::from(name),
            ),
          ))
        }
        SliceError::StatError(se) => ExpectPrimitiveError::UnderlyingProviderStatError(se),
      })?;

    if expect_fn(&v) {
      self.offset.set(PRIMITIVE_SIZE as u64 + self.offset());

      return Ok(v);
    };

    let dr = self.diagnostic_reference().create_physical_child(
      self.offset(),
      Some(PRIMITIVE_SIZE as u64),
      DiagnosticNodeName::from(name),
    );

    Err(ExpectPrimitiveError::ExpectationFailed(
      ExpectationFailedError(v, dr, error_fn),
    ))
  }

  pub fn seek(
    &mut self,
    position: SeekFrom,
  ) -> Result<(), SeekError<UnderlyingProvider::StatError>> {
    let len = self
      .provider
      .len()
      .map_err(|e| SeekError::UnderlyingProviderStatError(UnderlyingProviderStatError(e)))?;

    let (new_value, overflowed) = match position {
      SeekFrom::Current(bytes) => self.offset().overflowing_add_signed(bytes),
      SeekFrom::Start(bytes) => (bytes, false),
      SeekFrom::End(bytes) => len.overflowing_add_signed(bytes),
    };

    if overflowed || new_value >= len {
      Err(SeekError::Overflowed {
        available_size: len,
      })
    } else {
      self.offset.set(new_value);

      Ok(())
    }
  }

  pub fn with_dyn_bytes<T>(
    &self,
    length: Option<u64>,
    name: &str,
    callback: impl FnOnce(&[u8]) -> T,
  ) -> Result<
    T,
    ParsePrimitiveError<
      'pool,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    Ok(
      self
        .provider
        .with_read_dyn(self.offset(), length, callback)
        .map_err(|re| {
          ParsePrimitiveError::UnderlyingProviderReadError(UnderlyingProviderReadError(
            re.0,
            self.diagnostic_reference.create_physical_child(
              self.offset(),
              length,
              DiagnosticNodeName::from(name),
            ),
          ))
        })?
        .map_err(|e| match e {
          SliceError::OutOfBounds(oob) => {
            ParsePrimitiveError::OutOfBounds(ReadOutOfBoundsError::from_slice_out_of_bounds_error(
              oob,
              self.diagnostic_reference.create_physical_child(
                self.offset(),
                length,
                DiagnosticNodeName::from(name),
              ),
            ))
          }
          SliceError::StatError(se) => ParsePrimitiveError::UnderlyingProviderStatError(se),
        })?,
    )
  }

  pub fn with_dyn_bytes_at<T>(
    &self,
    offset: u64,
    length: Option<u64>,
    name: &str,
    callback: impl FnOnce(&[u8]) -> T,
  ) -> Result<
    T,
    ParsePrimitiveError<
      'pool,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    Ok(
      self
        .provider
        .with_read_dyn(offset, length, callback)
        .map_err(|re| {
          ParsePrimitiveError::UnderlyingProviderReadError(UnderlyingProviderReadError(
            re.0,
            self.diagnostic_reference.create_physical_child(
              offset,
              length,
              DiagnosticNodeName::from(name),
            ),
          ))
        })?
        .map_err(|e| match e {
          SliceError::OutOfBounds(oob) => {
            ParsePrimitiveError::OutOfBounds(ReadOutOfBoundsError::from_slice_out_of_bounds_error(
              oob,
              self.diagnostic_reference.create_physical_child(
                offset,
                length,
                DiagnosticNodeName::from(name),
              ),
            ))
          }
          SliceError::StatError(se) => ParsePrimitiveError::UnderlyingProviderStatError(se),
        })?,
    )
  }

  pub fn read<
    const TYPE_SIZE: usize,
    T: FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, TYPE_SIZE>,
  >(
    &self,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Result<
    T,
    ParseError<
      'pool,
      T::Error,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  >
  where
    T::Argument: NoneSizedArgument,
  {
    let dr = self.diagnostic_reference();

    let slice = self
      .provider
      .slice::<TYPE_SIZE>(self.offset())
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => ParseError::OutOfBounds(
          ReadOutOfBoundsError::from_slice_out_of_bounds_error(oob, dr),
        ),
        SliceError::StatError(se) => ParseError::UnderlyingProviderStatError(se),
      })?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: Cell::new(0),
      diagnostic_reference: dr.create_physical_child(self.offset(), Some(TYPE_SIZE as u64), name),
    };

    self.offset.set(TYPE_SIZE as u64 + self.offset());

    T::read(&mut child, T::Argument::from_none()).map_err(|e| match e {
      ParseError::DomainSpecific(ds) => ParseError::DomainSpecific(ds),
      ParseError::OutOfBounds(oob) => ParseError::OutOfBounds(oob),
      ParseError::UnderlyingProviderReadError(re) => ParseError::UnderlyingProviderReadError(re),
      ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(se)) => {
        ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(se))
      }
    })
  }

  pub fn read_with<
    const TYPE_SIZE: usize,
    T: FixedSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, TYPE_SIZE>,
  >(
    &self,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
    argument: T::Argument,
  ) -> Result<
    T,
    ParseError<
      'pool,
      T::Error,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let dr = self.diagnostic_reference();

    let slice = self
      .provider
      .slice::<TYPE_SIZE>(self.offset())
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => ParseError::OutOfBounds(
          ReadOutOfBoundsError::from_slice_out_of_bounds_error(oob, dr),
        ),
        SliceError::StatError(se) => ParseError::UnderlyingProviderStatError(se),
      })?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: Cell::new(0),
      diagnostic_reference: dr.create_physical_child(self.offset(), Some(TYPE_SIZE as u64), name),
    };

    self.offset.set(TYPE_SIZE as u64 + self.offset());

    T::read(&mut child, argument).map_err(|e| match e {
      ParseError::DomainSpecific(ds) => ParseError::DomainSpecific(ds),
      ParseError::OutOfBounds(oob) => ParseError::OutOfBounds(oob),
      ParseError::UnderlyingProviderReadError(re) => ParseError::UnderlyingProviderReadError(re),
      ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(se)) => {
        ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(se))
      }
    })
  }

  pub fn read_dyn<T: DynamicSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(
    &self,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Result<
    T,
    ParseError<
      'pool,
      T::Error,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  >
  where
    T::Argument: NoneSizedArgument,
  {
    let dr = self.diagnostic_reference();
    let size = T::get_size(self)?;

    let slice = self
      .provider
      .slice_dyn(self.offset(), size)
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => ParseError::OutOfBounds(
          ReadOutOfBoundsError::from_slice_out_of_bounds_error(oob, dr),
        ),
        SliceError::StatError(se) => ParseError::UnderlyingProviderStatError(se),
      })?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: Cell::new(0),
      diagnostic_reference: dr.create_physical_child(self.offset(), size, name),
    };

    let result = T::read(&mut child, T::Argument::from_none()).map_err(|e| match e {
      ParseError::DomainSpecific(ds) => ParseError::DomainSpecific(ds),
      ParseError::OutOfBounds(oob) => ParseError::OutOfBounds(oob),
      ParseError::UnderlyingProviderReadError(re) => ParseError::UnderlyingProviderReadError(re),
      ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(e)) => {
        ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(e))
      }
    });

    self.offset.set(child.offset() + self.offset());

    result
  }

  pub fn read_dyn_with<T: DynamicSizeReadable<'pool, DIAGNOSTIC_NODE_NAME_SIZE>>(
    &self,
    name: DiagnosticNodeName<DIAGNOSTIC_NODE_NAME_SIZE>,
    argument: T::Argument,
  ) -> Result<
    T,
    ParseError<
      'pool,
      T::Error,
      UnderlyingProvider::ReadError,
      UnderlyingProvider::StatError,
      DIAGNOSTIC_NODE_NAME_SIZE,
    >,
  > {
    let dr = self.diagnostic_reference();

    let size = T::get_size(self)?;

    let slice = self
      .provider
      .slice_dyn(self.offset(), size)
      .map_err(|e| match e {
        SliceError::OutOfBounds(oob) => ParseError::OutOfBounds(
          ReadOutOfBoundsError::from_slice_out_of_bounds_error(oob, dr),
        ),
        SliceError::StatError(se) => ParseError::UnderlyingProviderStatError(se),
      })?;

    let mut child = Reader {
      provider: slice,
      endianness: self.endianness,
      offset: Cell::new(0),
      diagnostic_reference: dr.create_physical_child(self.offset(), size, name),
    };

    let result = T::read(&mut child, argument).map_err(|e| match e {
      ParseError::DomainSpecific(ds) => ParseError::DomainSpecific(ds),
      ParseError::OutOfBounds(oob) => ParseError::OutOfBounds(oob),
      ParseError::UnderlyingProviderReadError(re) => ParseError::UnderlyingProviderReadError(re),
      ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(e)) => {
        ParseError::UnderlyingProviderStatError(UnderlyingProviderStatError(e))
      }
    });

    self.offset.set(child.offset() + self.offset());

    result
  }
}
