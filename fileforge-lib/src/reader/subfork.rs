use crate::{
  diagnostic::{node::name::DiagnosticNodeName, value::DiagnosticValue},
  stream::{error::stream_partition::StreamPartitionError, DynamicPartitionableStream, SkippableStream},
};

use super::{
  diagnostic_store::DiagnosticKind,
  error::{
    seek_out_of_bounds::{SeekOffset, SeekOutOfBounds},
    subfork::SubforkError,
  },
  Reader,
};

impl<'l, 'pool, const NODE_NAME_SIZE: usize, S: DynamicPartitionableStream<'l, NODE_NAME_SIZE>> Reader<'pool, NODE_NAME_SIZE, S>
where
  'pool: 'l,
{
  pub async fn subfork<'a>(
    &'a mut self,
    length: DiagnosticValue<'pool, u64, NODE_NAME_SIZE>,
    name: Option<impl Into<DiagnosticNodeName<NODE_NAME_SIZE>>>,
  ) -> Result<Reader<'pool, NODE_NAME_SIZE, S::PartitionDynamic>, SubforkError<'l, 'pool, NODE_NAME_SIZE, S>>
  where
    S: Clone,
    S: SkippableStream<NODE_NAME_SIZE>,
    'a: 'l,
  {
    let offset = self.stream.offset();

    let base = match self.stream.partition_dynamic(*length).await {
      Ok(v) => v,
      Err(StreamPartitionError::User(u)) => return Err(SubforkError::Stream(u)),
      Err(StreamPartitionError::StreamExhausted(se)) => {
        if let Some(value) = se.read_offset.checked_add(se.read_length) {
          return Err(SubforkError::OutOfBounds(SeekOutOfBounds {
            seek_offset: SeekOffset::InBounds(value),
            provider_size: self.diagnostics.infuse(DiagnosticKind::ReaderLength, se.stream_length),
            container_dr: self.diagnostics.get(DiagnosticKind::Reader),
          }));
        } else {
          return Err(SubforkError::OutOfBounds(SeekOutOfBounds {
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

    let mut reader = Reader::new(base, self.endianness);

    if let Some(reference) = self.diagnostics.get(DiagnosticKind::Reader) {
      if let Some(name) = name {
        reader.set_diagnostic(DiagnosticKind::Reader, reference.create_physical_child(offset, Some(*length), name));
      }
    }

    if let Some(reference) = length.reference() {
      reader.set_diagnostic(DiagnosticKind::ReaderLength, reference);
    }

    Ok(reader)
  }
}
