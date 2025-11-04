pub mod data;
pub mod error;

pub mod block_inflate_pair;
#[cfg(test)]
pub mod tests;

use std::num::NonZeroU16;

use fileforge_lib::{
  control_flow::ControlFlow,
  stream::{
    error::{stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError, stream_skip::StreamSkipError},
    MutableStream, ReadableStream, ResizableStream, RestorableStream, CLONED, DOUBLE, SINGLE,
  },
};

use crate::sead::yaz0::parser::{
  data::{Block, BlockHeader, Operation},
  error::{Component, Yaz0ParserError, Yaz0ParserMutateError, Yaz0ParserSkipError},
};

pub struct Yaz0Parser<UnderlyingStream: ReadableStream<Type = u8>> {
  underlying: UnderlyingStream,
  offset: u64,
  decoded_bytes_thusfar: u32,
  total_decoded_byte_length: u32,
}

impl<S: ReadableStream<Type = u8>> Yaz0Parser<S> {
  pub fn new(underlying: S, decoded_length: u32) -> Self {
    Self {
      underlying,
      offset: 0,
      decoded_bytes_thusfar: 0,
      total_decoded_byte_length: decoded_length,
    }
  }

  pub fn remaining_decoded_bytes(&self) -> u32 {
    self.total_decoded_byte_length.saturating_sub(self.decoded_bytes_thusfar)
  }
}

impl<S: ReadableStream<Type = u8>> ReadableStream for Yaz0Parser<S> {
  type Type = Block;

  type ReadError = Yaz0ParserError<S::ReadError>;
  type SkipError = Yaz0ParserSkipError<S::ReadError, S::SkipError>;

  fn offset(&self) -> u64 {
    self.offset
  }

  #[inline]
  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let mut collection = heapless::Vec::<Block, SIZE>::new();

    for _ in 0..SIZE {
      let mut header = BlockHeader::from_byte(self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::Header, e))?);
      let mut operations = heapless::Vec::<Operation, 8>::new();

      'block: for _ in 0..8 {
        if self.remaining_decoded_bytes() == 0 {
          break 'block;
        }

        let op = if header.take().unwrap() {
          Operation::Literal(match self.underlying.read(SINGLE).await {
            Ok(v) => v,
            Err(StreamReadError::StreamExhausted(_)) => break 'block,
            Err(StreamReadError::User(e)) => return Err(StreamReadError::User(Yaz0ParserError::ReadError(Component::Literal, e))),
          })
        } else {
          let b1: u8 = match self.underlying.read(SINGLE).await {
            Ok(v) => v,
            Err(StreamReadError::StreamExhausted(_)) => break 'block,
            Err(StreamReadError::User(e)) => return Err(StreamReadError::User(Yaz0ParserError::ReadError(Component::SequenceHeader, e))),
          };

          if (b1 & 0xF0) == 0 {
            // 3-byte form: 0R RR NN
            let (r_l, n): (u8, u8) = self.underlying.read(DOUBLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::LargeSequenceTail, e))?;
            let rrr = (((b1 as u16) << 8) | (r_l as u16)) as u16;
            Operation::LongReadback {
              offset: NonZeroU16::new(rrr + 1).unwrap(),
              length: NonZeroU16::new((n as u16) + 0x12).unwrap(),
            }
          } else {
            // 2-byte form: NR RR with N = b1>>4 (1..=0xF)
            let r_l: u8 = self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserError::ReadFailed(Component::SmallSequenceTail, e))?;
            let n = (b1 >> 4) as u16;
            let rrr = ((((b1 & 0x0F) as u16) << 8) | (r_l as u16)) as u16;
            Operation::ShortReadback {
              offset: NonZeroU16::new(rrr + 1).unwrap(),
              length: NonZeroU16::new(n + 2).unwrap(),
            }
          }
        };

        operations.push(op).ok().unwrap();
        self.decoded_bytes_thusfar += op.len() as u32;
      }

      collection.push(Block { operations }).ok().unwrap();
    }

    self.offset += SIZE as u64;
    Ok(reader(collection.first_chunk::<SIZE>().unwrap()).await)
  }

  async fn skip(&mut self, size: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    for _ in 0..size {
      let mut header = BlockHeader::from_byte(self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserSkipError::ReadFailed(Component::Header, e))?);

      'block: for _ in 0..8 {
        if header.take().unwrap() {
          self.underlying.skip(1).await.map_err(|e| Yaz0ParserSkipError::SkipFailed(Component::Literal, e))?;
          self.decoded_bytes_thusfar += 1;
        } else {
          let b1: u8 = match self.underlying.read(SINGLE).await {
            Ok(v) => v,
            Err(StreamReadError::StreamExhausted(_)) => break 'block,
            Err(StreamReadError::User(e)) => return Err(StreamSkipError::User(Yaz0ParserSkipError::ReadError(Component::SequenceHeader, e))),
          };

          let op = if (b1 & 0xF0) == 0 {
            // 3-byte form: 0R RR NN
            let (r_l, n): (u8, u8) = self.underlying.read(DOUBLE).await.map_err(|e| Yaz0ParserSkipError::ReadFailed(Component::LargeSequenceTail, e))?;
            let rrr = (((b1 as u16) << 8) | (r_l as u16)) as u16;
            Operation::LongReadback {
              offset: NonZeroU16::new(rrr + 1).unwrap(),
              length: NonZeroU16::new((n as u16) + 0x12).unwrap(),
            }
          } else {
            // 2-byte form: NR RR with N = b1>>4 (1..=0xF)
            let r_l: u8 = self.underlying.read(SINGLE).await.map_err(|e| Yaz0ParserSkipError::ReadFailed(Component::SmallSequenceTail, e))?;
            let n = (b1 >> 4) as u16;
            let rrr = ((((b1 & 0x0F) as u16) << 8) | (r_l as u16)) as u16;
            Operation::ShortReadback {
              offset: NonZeroU16::new(rrr + 1).unwrap(),
              length: NonZeroU16::new(n + 2).unwrap(),
            }
          };

          self.decoded_bytes_thusfar += op.len() as u32;
        };
      }
    }

    self.offset += size;
    Ok(())
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream + ResizableStream + MutableStream> Yaz0Parser<S> {
  async fn overwrite_operation(&mut self, len: u64, operation: Operation) -> Result<(), Yaz0ParserMutateError<S::ReadError, S::RestoreError, S::SkipError, S::OverwriteError, S::MutateError>> {
    Ok(match operation {
      Operation::ShortReadback { offset, length } => {
        let n = length.get() - 2;
        let r = offset.get() - 1;

        let b_0 = (n << 4 | r >> 8) as u8;
        let b_1 = (r & 0xFF) as u8;

        self.underlying.overwrite(len, [b_0, b_1]).await.map_err(|e| Yaz0ParserMutateError::OverwriteShortReadbackFailed(e))?
      }
      Operation::LongReadback { offset, length } => {
        let n = length.get() - 0x12;
        let r = offset.get() - 1;

        let b_0 = (r >> 8) as u8;
        let b_1 = (r & 0xFF) as u8;
        let b_2 = n as u8;

        self
          .underlying
          .overwrite(len, [b_0, b_1, b_2])
          .await
          .map_err(|e| Yaz0ParserMutateError::OverwriteShortReadbackFailed(e))?
      }
      Operation::Literal(new) => self.underlying.overwrite(len, [new]).await.map_err(|e| Yaz0ParserMutateError::OverwriteShortReadbackFailed(e))?,
    })
  }

  #[inline]
  async fn overwrite_block(&mut self, old: Option<&Block>, new: Option<&Block>) -> Result<(), Yaz0ParserMutateError<S::ReadError, S::RestoreError, S::SkipError, S::OverwriteError, S::MutateError>> {
    let original_operations = match old {
      Some(v) => Some(&v.operations),
      None => None,
    };
    let new_operations = match new {
      Some(v) => Some(&v.operations),
      None => None,
    };

    if let Some(original) = old {
      self.total_decoded_byte_length -= original.len() as u32;
    }

    if let Some(new) = new {
      self.total_decoded_byte_length += new.len() as u32;
      self.decoded_bytes_thusfar += new.len() as u32;
    }

    match (old, new) {
      (None, None) => return Ok(()),
      (Some(o), Some(n)) => {
        if n.compute_header() != o.compute_header() {
          self
            .underlying
            .mutate(async |v: &mut [u8; 1]| v[0] = n.compute_header())
            .await
            .map_err(|e| Yaz0ParserMutateError::MutateHeaderFailed(e))?;
        } else {
          self.underlying.skip(1).await.map_err(Yaz0ParserMutateError::SkipHeaderFailed)?;
        }
      }
      (Some(_), None) => {
        self.underlying.overwrite(1, []).await.map_err(|e| Yaz0ParserMutateError::RemoveHeaderFailed(e))?;
      }
      (None, Some(v)) => self.underlying.overwrite(0, [v.compute_header()]).await.map_err(|e| Yaz0ParserMutateError::CreateHeaderFailed(e))?,
    }

    if let Some(new_operations) = new_operations {
      for i in 0..8 {
        let original = original_operations.and_then(|v| v.get(i).copied());
        let new = new_operations.get(i).copied();

        match (original, new) {
          (None, None) => break,
          (None, Some(v)) => {
            self.overwrite_operation(0, v).await?;
          }
          (Some(o), Some(n)) => {
            if o != n {
              self.overwrite_operation(o.encoded_len() as u64, n).await?;
            } else {
              self.underlying.skip(o.encoded_len() as u64).await.map_err(|e| Yaz0ParserMutateError::SkipOperationFailed(e))?;
            }
          }
          (Some(o), None) if self.remaining_decoded_bytes() <= 0 => self
            .underlying
            .overwrite(o.encoded_len() as u64, [])
            .await
            .map_err(|e| Yaz0ParserMutateError::RemoveReadbackFailed(e))?,
          (Some(_), None) => Err(Yaz0ParserMutateError::ShrinkageBlocked)?,
        }
      }
    } else {
      for original in original_operations.iter().copied().flatten() {
        self
          .underlying
          .overwrite(original.encoded_len() as u64, [])
          .await
          .map_err(|e| Yaz0ParserMutateError::RemoveReadbackFailed(e))?;
      }
    }

    Ok(())
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream + ResizableStream + MutableStream> MutableStream for Yaz0Parser<S> {
  type MutateError = Yaz0ParserMutateError<S::ReadError, S::RestoreError, S::SkipError, S::OverwriteError, S::MutateError>;

  async fn mutate<const SIZE: usize, V: ControlFlow>(
    &mut self,
    mutator: impl AsyncFnOnce(&mut [Self::Type; SIZE]) -> V,
  ) -> Result<V, fileforge_lib::stream::error::stream_mutate::StreamMutateError<Self::MutateError>> {
    let snapshot = self.underlying.snapshot();
    let snapshot_len = self.decoded_bytes_thusfar;

    let (original_values, new_values, result) = self
      .read(async |original_values: &[Self::Type; SIZE]| {
        let mut new_values = original_values.clone();
        let result = mutator(&mut new_values).await;

        (original_values.clone(), new_values, result)
      })
      .await
      .map_err(|e| match e {
        StreamReadError::StreamExhausted(e) => StreamMutateError::StreamExhausted(e),
        StreamReadError::User(u) => StreamMutateError::User(Yaz0ParserMutateError::ReadFailed(u)),
      })?;

    if !result.should_continue() {
      return Ok(result);
    }

    self.underlying.restore(snapshot).await.map_err(|e| Yaz0ParserMutateError::RestoreFailed(e))?;
    self.decoded_bytes_thusfar = snapshot_len;

    for (original_block, new_block) in original_values.iter().zip(new_values.iter()) {
      self.overwrite_block(Some(original_block), Some(new_block)).await?
    }

    Ok(result)
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream + ResizableStream + MutableStream> ResizableStream for Yaz0Parser<S> {
  type OverwriteError = Yaz0ParserMutateError<S::ReadError, S::RestoreError, S::SkipError, S::OverwriteError, S::MutateError>;

  async fn overwrite<const SIZE: usize>(&mut self, mut length: u64, data: [Self::Type; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    let mut current_data = &data[..];

    while length != 0 {
      let snapshot = self.underlying.snapshot();
      let thusfar = self.decoded_bytes_thusfar;

      let original_values = self.read(CLONED).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(e) => StreamOverwriteError::StreamExhausted(e),
        StreamReadError::User(u) => StreamOverwriteError::User(Yaz0ParserMutateError::ReadFailed(u)),
      })?;

      let data = current_data.split_first().map(|(first, remaining)| {
        current_data = remaining;
        first
      });

      self.underlying.restore(snapshot).await.map_err(|e| Yaz0ParserMutateError::RestoreFailed(e))?;
      self.decoded_bytes_thusfar = thusfar;

      self.overwrite_block(Some(&original_values), data).await?;

      length -= 1;
    }

    for block in current_data {
      self.overwrite_block(None, Some(block)).await?;
    }

    self.offset += data.len() as u64;
    Ok(())
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream> RestorableStream for Yaz0Parser<S> {
  type RestoreError = S::RestoreError;
  type Snapshot = (u64, u32, S::Snapshot);

  fn snapshot(&self) -> Self::Snapshot {
    (self.offset, self.decoded_bytes_thusfar, self.underlying.snapshot())
  }

  async fn restore(&mut self, snapshot: Self::Snapshot) -> Result<(), StreamRestoreError<Self::RestoreError>> {
    if snapshot.0 <= self.offset {
      self.underlying.restore(snapshot.2).await?;
      self.decoded_bytes_thusfar = snapshot.1;
      self.offset = snapshot.0;
      Ok(())
    } else {
      Err(StreamRestoreError::CannotRestoreForwards)
    }
  }
}
