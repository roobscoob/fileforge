use crate::{
  diagnostic::value::DiagnosticValue,
  stream::{error::stream_partition::StreamPartitionError, DynamicPartitionableStream, ReadableStream, StaticPartitionableStream},
};

use super::{
  diagnostic_store::DiagnosticKind,
  error::{
    dynamic_subfork::DynamicSubforkError,
    seek_out_of_bounds::{SeekOffset, SeekOutOfBounds},
    static_subfork::StaticSubforkError,
  },
  BinaryReader,
};

impl<'l, 'pool, S: DynamicPartitionableStream<'l, Type = u8>> BinaryReader<'pool, S>
where
  'pool: 'l,
{
  pub async fn subfork_dynamic<'a>(&'a mut self, length: DiagnosticValue<'pool, u64>, name: Option<&str>) -> Result<BinaryReader<'pool, S::PartitionDynamic>, DynamicSubforkError<'l, 'pool, S>>
  where
    'a: 'l,
  {
    let offset = self.stream.offset();

    let base = match self.stream.partition_dynamic(*length).await {
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

    let mut reader = BinaryReader::new(base, self.endianness);

    if let Some(reference) = self.diagnostics.get(DiagnosticKind::Reader) {
      if let Some(name) = name {
        reader.set_diagnostic(DiagnosticKind::Reader, Some(reference.create_physical_child(offset, Some(*length), name)));
      }
    }

    if let Some(reference) = length.reference() {
      reader.set_diagnostic(DiagnosticKind::ReaderLength, Some(reference));
    }

    Ok(reader)
  }
}

impl<'pool, S: ReadableStream<Type = u8>> BinaryReader<'pool, S> {
  pub async fn subfork_static<'a, const SIZE: usize>(&'a mut self, name: Option<&str>) -> Result<BinaryReader<'pool, S::Partition<'a>>, StaticSubforkError<'pool, SIZE, S>>
  where
    S: StaticPartitionableStream<SIZE>,
  {
    let offset = self.stream.offset();

    let base = match self.stream.partition().await {
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

    let mut reader = BinaryReader::new(base, self.endianness);

    if let Some(reference) = self.diagnostics.get(DiagnosticKind::Reader) {
      if let Some(name) = name {
        reader.set_diagnostic(DiagnosticKind::Reader, Some(reference.create_physical_child(offset, Some(SIZE as u64), name)));
      }
    }

    Ok(reader)
  }
}
