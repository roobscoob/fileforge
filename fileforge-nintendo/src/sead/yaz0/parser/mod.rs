pub mod data;
pub mod error;

use std::num::NonZeroU16;

use fileforge_lib::stream::{
  error::{stream_read::StreamReadError, stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError},
  MutableStream, ReadableStream, ResizableStream, RestorableStream, RewindableStream, DOUBLE, SINGLE,
};

use crate::sead::yaz0::parser::{
  data::{BlockHeader, Operation},
  error::{Component, Yaz0ParserError, Yaz0ParserMutateError, Yaz0ParserSkipError},
};

trait MaybeSnapshotData<S: ReadableStream<Type = u8>> {
  fn perform_pre_header_snapshot(&mut self, stream: &S);
}

trait SnapshotData<S: RestorableStream<Type = u8>>: MaybeSnapshotData<S> {
  fn pre_header_snapshot(&mut self) -> &mut Option<S::Snapshot>;
}

struct ConcreteSnapshotData<S: RestorableStream> {
  snapshot: Option<S::Snapshot>,
}

pub struct NoSnapshotData;

impl<S: RestorableStream<Type = u8>> MaybeSnapshotData<S> for ConcreteSnapshotData<S> {
  #[inline]
  fn perform_pre_header_snapshot(&mut self, stream: &S) {
    self.snapshot = Some(stream.snapshot())
  }
}

impl<S: RestorableStream<Type = u8>> SnapshotData<S> for ConcreteSnapshotData<S> {
  #[inline]
  fn pre_header_snapshot(&mut self) -> &mut Option<S::Snapshot> {
    &mut self.snapshot
  }
}

impl<S: ReadableStream<Type = u8>> MaybeSnapshotData<S> for NoSnapshotData {
  #[inline]
  fn perform_pre_header_snapshot(&mut self, _: &S) {}
}

pub struct Yaz0Parser<UnderlyingStream: ReadableStream<Type = u8>, SnapshotData: MaybeSnapshotData<UnderlyingStream>> {
  underlying: UnderlyingStream,
  current_header: BlockHeader,
  header_distance: u8,
  offset: u64,

  snapshot_data: SnapshotData,
}

impl<S: ReadableStream<Type = u8>> Yaz0Parser<S, NoSnapshotData> {
  pub fn new(underlying: S) -> Self {
    Self {
      underlying,
      current_header: BlockHeader::empty(),
      header_distance: 0,
      offset: 0,
      snapshot_data: NoSnapshotData,
    }
  }
}

impl<S: RestorableStream<Type = u8>> Yaz0Parser<S, ConcreteSnapshotData<S>> {
  pub fn new_with_snapshots(underlying: S, snapshot: Option<S::Snapshot>) -> Self {
    Self {
      underlying,
      current_header: BlockHeader::empty(),
      header_distance: 0,
      offset: 0,
      snapshot_data: ConcreteSnapshotData { snapshot },
    }
  }
}

impl<S: ReadableStream<Type = u8>, D: MaybeSnapshotData<S>> ReadableStream for Yaz0Parser<S, D> {
  type Type = Operation;

  type ReadError = Yaz0ParserError<S::ReadError>;
  type SkipError = Yaz0ParserSkipError<S::ReadError, S::SkipError>;

  fn offset(&self) -> u64 {
    self.offset
  }

  #[inline]
  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let mut collection = heapless::Vec::<Operation, SIZE>::new();

    for _ in 0..SIZE {
      self.header_distance += 1;
      let next_is_literal = match self.current_header.take() {
        Some(v) => v,
        None => {
          self.snapshot_data.perform_pre_header_snapshot(&self.underlying);
          self.current_header = BlockHeader::from_byte(self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::Header, e))?);
          self.header_distance = 0;
          self.current_header.take().unwrap()
        }
      };

      let op = if next_is_literal {
        Operation::Literal(self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::Literal, e))?)
      } else {
        let b1: u8 = self.underlying.read(SINGLE::<u8>).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::SequenceHeader, e))?;

        if (b1 & 0xF0) == 0 {
          // 3-byte form: 0R RR NN
          let (r_l, n): (u8, u8) = self.underlying.read(DOUBLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::LargeSequenceTail, e))?;
          let rrr = (((b1 as u16) << 8) | (r_l as u16)) as u16;
          Operation::Readback {
            offset: rrr + 1,
            length: NonZeroU16::new((n as u16) + 0x12).unwrap(),
          }
        } else {
          // 2-byte form: NR RR with N = b1>>4 (1..=0xF)
          let r_l: u8 = self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::SmallSequenceTail, e))?;
          let n = (b1 >> 4) as u16;
          let rrr = ((((b1 & 0x0F) as u16) << 8) | (r_l as u16)) as u16;
          Operation::Readback {
            offset: rrr + 1,
            length: NonZeroU16::new(n + 2).unwrap(),
          }
        }
      };

      collection.push(op).ok().unwrap();
      self.offset += 1;
    }

    Ok(reader(collection.first_chunk::<SIZE>().unwrap()).await)
  }

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    for _ in 0..size {
      self.header_distance += 1;
      let next_is_literal = match self.current_header.take() {
        Some(v) => v,
        None => {
          self.snapshot_data.perform_pre_header_snapshot(&self.underlying);
          self.current_header = BlockHeader::from_byte(self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserSkipError::ReadFailed(Component::Header, e))?);
          self.header_distance = 0;
          self.current_header.take().unwrap()
        }
      };

      if next_is_literal {
        self.underlying.skip(1).await.map_err(|e| Yaz0ParserSkipError::SkipFailed(Component::Literal, e))?
      } else {
        let b1: u8 = self.underlying.read(SINGLE::<u8>).await.map_err(|e| Yaz0ParserSkipError::ReadFailed(Component::SequenceHeader, e))?;

        self
          .underlying
          .skip(if (b1 & 0xF0) == 0 { 2 } else { 1 })
          .await
          .map_err(|e| Yaz0ParserSkipError::SkipFailed(if (b1 & 0xF0) == 0 { Component::LargeSequenceTail } else { Component::SmallSequenceTail }, e))?
      };

      self.offset += 1;
    }

    Ok(())
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream + ResizableStream, D: SnapshotData<S>> MutableStream for Yaz0Parser<S, D> {
  type MutateError = Yaz0ParserMutateError;

  async fn mutate<const SIZE: usize, V>(
    &mut self,
    mutator: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V,
  ) -> Result<V, fileforge_lib::stream::error::stream_mutate::StreamMutateError<Self::MutateError>> {
  }
}
