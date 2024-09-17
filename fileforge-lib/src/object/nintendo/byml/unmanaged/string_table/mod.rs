use intx::U24;

use crate::{provider::r#trait::Provider, reader::{Reader, SeekFrom, error::{ParsePrimitiveErrorResultExtension, ExpectPrimitiveErrorResultExtension}}, error::render::{builtin::{text::Text, number::formatted_unsigned::FormattedUnsigned}, buffer::cell::tag::{builtin::report::REPORT_ERROR_TEXT, CellTag}, r#trait::renderable::Renderable}, diagnostic::node::name::DiagnosticNodeName};

use self::error::StringTableSizeError;

pub mod error;

pub struct StringTable {
}

impl StringTable {
  pub fn size<'pool, const DIAGNOSTIC_NODE_NAME_SIZE: usize, P: Provider>(reader: &mut Reader<'pool, DIAGNOSTIC_NODE_NAME_SIZE, P>) -> Result<u64, StringTableSizeError<'pool, P::ReadError, impl for <'primitive, 'closure> Fn(&'primitive u8, &'closure mut dyn for <'text, 'tag> FnMut(Text<'text, 'tag>, &'tag dyn CellTag, Option<&'text dyn Renderable<'tag>>) -> ()) -> (), DIAGNOSTIC_NODE_NAME_SIZE>>{
    let kind = reader.expect("Node Kind", |kind: &u8| { *kind == 0xC2 || *kind == 0xC5 }, |primitive, cb| {
      let value_base_16 = FormattedUnsigned::new(*primitive as u64).with_base(16).with_uppercase().with_padding(2);

      let text = Text::new().push("Expected kind to be a valid string table (0xC2 or 0xC5). Instead it was ", &REPORT_ERROR_TEXT)
        .with(&value_base_16);

      cb(text, &REPORT_ERROR_TEXT, Some(&value_base_16));
    })
      .map_expectation_failed(|ef| StringTableSizeError::InvalidNodeKind(ef))?
      .map_out_of_bounds(|oob| StringTableSizeError::MissingNodeKind(oob))??;

    let count: U24 = reader.get("Element Count")
      .map_out_of_bounds(|oob| StringTableSizeError::MissingElementCount(oob))??;

    // if it is a *RELOCATED* string table (because those exist...)
    if kind == 0xC5 {
      let expected_remaining = 0x4 * Into::<u64>::into(count);
      let remaining = reader.remaining();

      if remaining < expected_remaining {
        reader.seek(SeekFrom::Current(-3)).expect("to be able to move 3 bytes backward (U24)");

        return Err(StringTableSizeError::NotLargeEnough {
          expectation_value: expected_remaining,
          expectation_source_value: Into::<u64>::into(count),
          expectation_source_reference: reader.diagnostic_reference().create_physical_child(reader.offset(), 3, DiagnosticNodeName::from("Element Count")),
          actual_value: remaining,
        });
      }

      return Ok(expected_remaining + 0x4);
    }

    let remaining = reader.remaining();
    let source = Into::<u32>::into(count);
    let expectation = source * 0x4;

    reader.seek(SeekFrom::Current(expectation as i64))
      .map_err(|_| {
        reader.seek(SeekFrom::Current(-3)).expect("to be able to move 3 bytes backward (U24)");

        StringTableSizeError::NotLargeEnough {
          expectation_value: (expectation as u64) + 4,
          actual_value: remaining,
          expectation_source_reference: reader.diagnostic_reference().create_physical_child(reader.offset(), 3, DiagnosticNodeName::from("Element Count")),
          expectation_source_value: source as u64,
        }
      })?;

    let size: u32 = reader.get("Address Table (Last Element)")
      .map_out_of_bounds(|_| {
        reader.seek(SeekFrom::Current(-3)).expect("to be able to move 3 bytes backward (U24)");

        StringTableSizeError::NotLargeEnough {
          expectation_source_reference: reader.diagnostic_reference().create_physical_child(reader.offset(), 3, DiagnosticNodeName::from("Element Count")),
          expectation_source_value: source as u64,
          expectation_value: (expectation as u64) + 4,
          actual_value: remaining,
        }
      })??;
      
    Ok(size as u64)
  }
}