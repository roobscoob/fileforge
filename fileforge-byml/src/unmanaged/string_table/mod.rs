use core::ffi::CStr;

use error::{
  get::GetError,
  get_length::{GetLengthError, StringTableNotLargeEnough},
  get_offset::GetOffsetError,
};

use intx::U24;

use fileforge_lib::{
  diagnostic::node::{name::DiagnosticNodeName, reference::DiagnosticReference},
  error::render::{
    buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT,
    builtin::{
      byte_display::ByteDisplay, number::formatted_unsigned::FormattedUnsigned, text::Text,
    },
  },
  provider::{error::ProviderError, r#trait::Provider},
  reader::{
    error::{
      expect_primitive::ExpectPrimitiveErrorResultExtension,
      parse_primitive::{ParsePrimitiveError, ParsePrimitiveErrorResultExtension},
      underlying_provider_error::UnderlyingProviderError,
      underlying_provider_stat::UnderlyingProviderStatError,
    },
    Reader, SeekFrom,
  },
};

use crate::util::binary_search::fallible_binary_search;

use self::error::size::StringTableSizeError;

use super::error::get_string_table::StringTableOutOfBounds;

pub mod error;

pub struct StringTable<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider> {
  pub(super) reader: Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>,
}

pub struct DiagnosticReferenceBuilder<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize> {
  real_value: u32,
  real_value_size: Option<u64>,
  parent_dr: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  real_value_offset: u64,
  real_value_name: &'static str,
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize>
  DiagnosticReferenceBuilder<'pool, DIAGNOSTIC_NODE_NAME_SIZE>
{
  pub fn build_dr(&self) -> (u32, DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>) {
    (
      self.real_value,
      self.parent_dr.create_physical_child(
        self.real_value_offset,
        self.real_value_size,
        DiagnosticNodeName::from(self.real_value_name),
      ),
    )
  }

  pub fn relocate_build(
    &self,
    dr: DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Option<(u32, DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>)> {
    dr.dereference().map(|node| {
      (
        self.real_value,
        dr.create_physical_child(
          self.real_value_offset - node.branch.offset().unwrap(),
          self.real_value_size,
          DiagnosticNodeName::from(self.real_value_name),
        ),
      )
    })
  }
}

impl<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider>
  StringTable<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>
{
  pub fn size(
    reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>,
    get_byml_size: impl Fn() -> Result<u64, UnderlyingProviderStatError<P::StatError>>,
    get_position_dr: impl Fn() -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
  ) -> Result<
    (
      u64,
      DiagnosticReferenceBuilder<'pool, DIAGNOSTIC_NODE_NAME_SIZE>,
    ),
    StringTableSizeError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE>,
  > {
    let kind = reader
      .expect(
        "Node Kind",
        |kind: &u8| *kind == 0xC2 || *kind == 0xC5,
        |primitive, cb| {
          let value_base_16 = FormattedUnsigned::new(*primitive as u64)
            .with_base(16)
            .with_uppercase()
            .with_padding(2);

          let text = Text::new()
            .push(
              "Expected kind to be a valid string table (0xC2 or 0xC5). Instead it was ",
              &REPORT_ERROR_TEXT,
            )
            .with(&value_base_16);

          cb(text, &REPORT_ERROR_TEXT, Some(&value_base_16));
        },
      )
      .map_expectation_failed(|ef| StringTableSizeError::InvalidNodeKind(ef))?
      .map_out_of_bounds(|oob| {
        StringTableSizeError::NotLargeEnough(StringTableOutOfBounds {
          byml_size: get_byml_size(),
          string_table_parent: reader.diagnostic_reference(),
          string_table_position: reader.offset() as usize,
          string_table_position_dr: get_position_dr(),
          string_table_size: Some(4usize),
          string_table_size_complete: false,
          string_table_size_dr: None,
        })
      })??;

    let count: U24 = reader.get("Element Count").map_out_of_bounds(|oob| {
      StringTableSizeError::NotLargeEnough(StringTableOutOfBounds {
        byml_size: get_byml_size(),
        string_table_parent: reader.diagnostic_reference(),
        string_table_position: (reader.offset() - 1) as usize,
        string_table_position_dr: get_position_dr(),
        string_table_size: Some(4usize),
        string_table_size_complete: false,
        string_table_size_dr: None,
      })
    })??;

    // if it is a *RELOCATED* string table (because those exist...)
    if kind == 0xC5 {
      let expected_remaining = 0x4 * Into::<u64>::into(count);
      let remaining = reader.remaining();

      match remaining {
        Ok(remaining) => {
          if remaining < expected_remaining {
            reader
              .seek(SeekFrom::Current(-3))
              .unwrap_or_else(|_| panic!("to be able to move 3 bytes backward (U24)"));
            return Err(StringTableSizeError::NotLargeEnough(
              StringTableOutOfBounds {
                byml_size: get_byml_size(),
                string_table_parent: reader.diagnostic_reference(),
                string_table_position: (reader.offset() - 1) as usize,
                string_table_position_dr: get_position_dr(),
                string_table_size: Some((expected_remaining + 4) as usize),
                string_table_size_complete: true,
                string_table_size_dr: Some(DiagnosticReferenceBuilder {
                  parent_dr: reader.diagnostic_reference(),
                  real_value: Into::<u32>::into(count),
                  real_value_name: "Element Count",
                  real_value_offset: reader.offset(),
                  real_value_size: Some(3),
                }),
              },
            ));
          }
        }
        Err(stat_error) => {
          return Err(StringTableSizeError::UnderlyingProviderError(
            UnderlyingProviderError::StatError(UnderlyingProviderStatError(stat_error)),
          ))
        }
      };

      return Ok((
        expected_remaining + 0x4,
        DiagnosticReferenceBuilder {
          real_value: Into::<u32>::into(count),
          parent_dr: reader.diagnostic_reference(),
          real_value_offset: reader.offset() - 3,
          real_value_name: "Element Count",
          real_value_size: Some(3),
        },
      ));
    }

    let source = Into::<u32>::into(count);

    let size: u32 = reader
      .get_at("Address Table (Last Element)", ((source * 0x4) + 4) as u64)
      .map_out_of_bounds(|_| {
        reader
          .seek(SeekFrom::Current(-3 + -((source * 0x4) as i64)))
          .unwrap_or_else(|_| panic!("to be able to move 3 bytes backward (U24)"));

        return StringTableSizeError::NotLargeEnough(StringTableOutOfBounds {
          byml_size: get_byml_size(),
          string_table_parent: reader.diagnostic_reference(),
          string_table_position: (reader.offset() - 1) as usize,
          string_table_position_dr: get_position_dr(),
          string_table_size: Some(((source * 0x4) + 4) as usize),
          string_table_size_complete: false,
          string_table_size_dr: Some(DiagnosticReferenceBuilder {
            parent_dr: reader.diagnostic_reference(),
            real_value: Into::<u32>::into(count),
            real_value_name: "Element Count",
            real_value_offset: reader.offset(),
            real_value_size: Some(3),
          }),
        });
      })??;

    Ok((
      size as u64,
      DiagnosticReferenceBuilder {
        parent_dr: reader.diagnostic_reference(),
        real_value: size,
        real_value_name: "Offset Table (Last Element)",
        real_value_offset: reader.offset() - 4,
        real_value_size: Some(3),
      },
    ))
  }

  pub fn length(&mut self) -> Result<u32, GetLengthError<'pool, P, DIAGNOSTIC_NODE_NAME_SIZE>> {
    Ok(
      self
        .reader
        .get_at::<3, U24>("Length", 1)
        .map_err(|_| {
          GetLengthError::NotLargeEnough(StringTableNotLargeEnough {
            desired_length: Some(4),
            available_length: self.reader.len(),
          })
        })?
        .into(),
    )
  }

  fn length_diagnostic_reference(&self) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.reader.diagnostic_reference().create_physical_child(
      1,
      Some(3),
      DiagnosticNodeName::from("Length"),
    )
  }

  fn offset_table_diagnostic_reference(
    &self,
    length: u32,
  ) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.reader.diagnostic_reference().create_physical_child(
      4,
      Some(length as u64 * 4),
      DiagnosticNodeName::from("Offset Table"),
    )
  }

  fn string_pool_diagnostic_reference(
    &self,
    length: u32,
  ) -> DiagnosticReference<'pool, DIAGNOSTIC_NODE_NAME_SIZE> {
    self.reader.diagnostic_reference().create_physical_child(
      4 + (length as u64 * 4),
      None,
      DiagnosticNodeName::from("String Pool"),
    )
  }

  fn try_get_offset(
    &mut self,
    index: u32,
    length: Option<u32>,
  ) -> Result<(u32, u32), GetOffsetError<'pool, P, DIAGNOSTIC_NODE_NAME_SIZE>> {
    let length = match length {
      Some(v) => v,
      None => self.length().map_err(|e| match e {
        GetLengthError::NotLargeEnough(nle) => GetOffsetError::NotLargeEnough(nle),
        GetLengthError::UnderlyingProviderError(upe) => {
          GetOffsetError::UnderlyingProviderError(upe)
        }
      })?,
    };

    if index >= length {
      return Err(GetOffsetError::IndexOutOfBounds {
        requested_index: index,
        length_value: length,
        length_dr: self.length_diagnostic_reference(),
      });
    };

    let offset: u32 = self
      .reader
      .get_at("Offset", (4 + (index * 4)) as u64)
      .map_out_of_bounds(|e| {
        GetOffsetError::NotLargeEnough(StringTableNotLargeEnough {
          desired_length: e.read_size.map(|read_size| e.read_offset + read_size),
          available_length: Ok(e.reader_size),
        })
      })?
      .map_err(|e| GetOffsetError::UnderlyingProviderError(e))?;

    Ok((offset, length))
  }

  pub fn try_get<T>(
    &mut self,
    index: u32,
    cb: impl for<'s> FnOnce(&'s CStr) -> T,
  ) -> Result<T, GetError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE, 128>> {
    println!(
      "!!! Hello: {} {:?}",
      index,
      self.reader.diagnostic_reference()
    );

    let (offset, length) = self.try_get_offset(index, None).map_err(|e| match e {
      GetOffsetError::IndexOutOfBounds {
        requested_index,
        length_dr,
        length_value,
      } => GetError::IndexOutOfBounds {
        requested_index,
        length_dr,
        length_value,
      },
      GetOffsetError::UnderlyingProviderError(upe) => GetError::UnderlyingProviderError(upe),
      GetOffsetError::NotLargeEnough(nle) => GetError::NotLargeEnough(nle),
    })?;

    let next_offset = match self.try_get_offset(index + 1, Some(length)) {
      Ok((offset, _)) => Some(offset),
      Err(GetOffsetError::NotLargeEnough(nle)) => return Err(GetError::NotLargeEnough(nle)),
      Err(GetOffsetError::UnderlyingProviderError(upe)) => {
        return Err(GetError::UnderlyingProviderError(upe))
      }
      Err(GetOffsetError::IndexOutOfBounds { .. }) => None,
    };

    let string_length_upper_bound = next_offset.map(|next_offset| (next_offset - offset) as u64);

    let x = self
      .reader
      .with_dyn_bytes_at(offset as u64, string_length_upper_bound, "Name", |bytes| {
        Ok(cb(
          CStr::from_bytes_until_nul(bytes).map_err(|e| (e, ByteDisplay::new(bytes)))?,
        ))
      })
      .map_err(|e| match e {
        ParsePrimitiveError::OutOfBounds(e) => {
          GetError::NotLargeEnough(StringTableNotLargeEnough {
            desired_length: Some(e.read_offset),
            available_length: self.reader.len(),
          })
        }
        ParsePrimitiveError::UnderlyingProviderReadError(upre) => {
          GetError::UnderlyingProviderError(UnderlyingProviderError::ReadError(upre))
        }
        ParsePrimitiveError::UnderlyingProviderStatError(upse) => {
          GetError::UnderlyingProviderError(UnderlyingProviderError::StatError(upse))
        }
      })?
      .map_err(|(e, display)| {
        let string_pool = self.string_pool_diagnostic_reference(length);

        GetError::CStrError(
          e,
          string_pool.create_physical_child(
            offset as u64,
            string_length_upper_bound,
            DiagnosticNodeName::from_index(index as u64),
          ),
          display,
        )
      })?;

    Ok(x)
  }

  pub fn try_get_index(
    &mut self,
    needle: &CStr,
  ) -> Result<
    Option<u32>,
    GetError<'pool, P::ReadError, P::StatError, DIAGNOSTIC_NODE_NAME_SIZE, 128>,
  > {
    let length = self.length().map_err(|e| match e {
      GetLengthError::NotLargeEnough(nle) => GetError::NotLargeEnough(nle),
      GetLengthError::UnderlyingProviderError(upe) => GetError::UnderlyingProviderError(upe),
    })?;

    if length == 0 {
      return Ok(None);
    }

    fallible_binary_search(length as usize, |index| {
      self.try_get(index as u32, |string| needle.cmp(string))
    })
    .map(|v| v.map(|v| v as u32))
  }
}
