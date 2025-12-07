use crate::{
  binary_reader::error::{common::SeekOffset, DynamicSubforkError, StaticSubforkError},
  diagnostic::value::DiagnosticValue,
  stream::{error::stream_partition::StreamPartitionError, DynamicPartitionableStream, ReadableStream, StaticPartitionableStream},
};

use super::{diagnostic_store::DiagnosticKind, error::seek_out_of_bounds::SeekOutOfBounds, BinaryReader};

impl<'pool, S: DynamicPartitionableStream<Type = u8>> BinaryReader<'pool, S> {
  pub async fn partition_dynamic<'a>(
    self,
    length: impl Into<DiagnosticValue<'pool, u64>>,
    name: Option<&str>,
  ) -> Result<(BinaryReader<'pool, S::PartitionDynamicLeft>, BinaryReader<'pool, S::PartitionDynamicRight>), DynamicSubforkError<'pool, S::PartitionError>> {
    let length = length.into();
    let offset = self.stream.offset();

    let (left, right) = match self.stream.partition_dynamic(*length).await {
      Ok(v) => v,
      Err(StreamPartitionError::User(u)) => return Err(DynamicSubforkError::Stream(u)),
      Err(StreamPartitionError::StreamExhausted(se)) => {
        if let Some(value) = se.read_offset.checked_add(se.read_length) {
          return Err(DynamicSubforkError::OutOfBounds(SeekOutOfBounds {
            seek_offset: SeekOffset::InBounds(value),
            provider_size: self.diagnostics.infuse(DiagnosticKind::ReaderLength, se.stream_length),
            container_dr: self.diagnostics.get(DiagnosticKind::Reader),
          }));
        } else {
          return Err(DynamicSubforkError::OutOfBounds(SeekOutOfBounds {
            seek_offset: SeekOffset::Overflowed {
              base_offset: se.read_offset,
              add: se.read_length,
            },
            container_dr: self.diagnostics.get(DiagnosticKind::Reader),
            provider_size: self.diagnostics.infuse(DiagnosticKind::ReaderLength, se.stream_length),
          }));
        }
      }
    };

    let mut left = BinaryReader::new(left, self.endianness);

    left.base_offset = self.base_offset;

    let mut right = BinaryReader::new(right, self.endianness);

    right.base_offset = self.base_offset + *length;
    right.diagnostics = self.diagnostics;

    if let Some(reference) = self.diagnostics.get(DiagnosticKind::Reader) {
      if let Some(name) = name {
        left.set_diagnostic(DiagnosticKind::Reader, Some(reference.create_physical_child(offset, Some(*length), name)));
      }
    }

    if let Some(reference) = length.reference() {
      left.set_diagnostic(DiagnosticKind::ReaderLength, Some(reference));
    }

    Ok((left, right))
  }
}

impl<'pool, S: ReadableStream<Type = u8>> BinaryReader<'pool, S> {
  pub async fn partition<'a, const SIZE: usize>(
    self,
    name: Option<&str>,
  ) -> Result<(BinaryReader<'pool, S::PartitionLeft>, BinaryReader<'pool, S::PartitionRight>), StaticSubforkError<'pool, S::PartitionError>>
  where
    S: StaticPartitionableStream<SIZE>,
  {
    let offset = self.stream.offset();

    let (left, right) = match self.stream.partition().await {
      Ok(v) => v,
      Err(StreamPartitionError::User(u)) => return Err(StaticSubforkError::Stream(u)),
      Err(StreamPartitionError::StreamExhausted(se)) => {
        if let Some(value) = se.read_offset.checked_add(se.read_length) {
          return Err(StaticSubforkError::OutOfBounds(SeekOutOfBounds {
            seek_offset: SeekOffset::InBounds(value),
            provider_size: self.diagnostics.infuse(DiagnosticKind::ReaderLength, se.stream_length),
            container_dr: self.diagnostics.get(DiagnosticKind::Reader),
          }));
        } else {
          return Err(StaticSubforkError::OutOfBounds(SeekOutOfBounds {
            seek_offset: SeekOffset::Overflowed {
              base_offset: se.read_offset,
              add: se.read_length,
            },
            container_dr: self.diagnostics.get(DiagnosticKind::Reader),
            provider_size: self.diagnostics.infuse(DiagnosticKind::ReaderLength, se.stream_length),
          }));
        }
      }
    };

    let mut left = BinaryReader::new(left, self.endianness);

    left.base_offset = self.base_offset;

    let mut right = BinaryReader::new(right, self.endianness);

    right.base_offset = self.base_offset + SIZE as u64;
    right.diagnostics = self.diagnostics;

    if let Some(reference) = self.diagnostics.get(DiagnosticKind::Reader) {
      if let Some(name) = name {
        left.set_diagnostic(DiagnosticKind::Reader, Some(reference.create_physical_child(offset, Some(SIZE as u64), name)));
      }
    }

    Ok((left, right))
  }
}
