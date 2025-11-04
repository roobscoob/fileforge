use std::fmt::Debug;

use fileforge_lib::stream::{
  error::{
    stream_exhausted::StreamExhaustedError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError,
    stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError,
  },
  MutableStream, ReadableStream, ResizableStream, RestorableStream, CLONED,
};

use crate::sead::yaz0::{
  error::{overwrite::Yaz0OverwriteError, Yaz0Error},
  header::Yaz0Header,
  parser::{
    block_inflate_pair::inflate_pair,
    data::{Block, Operation},
    Yaz0Parser,
  },
  readable::Yaz0StreamReadArgument,
  state::{malformed_stream::MalformedStream, reference::ReadbackReference, Yaz0State},
  store::{MaybeSnapshotStore, SnapshotStore},
};

pub mod error;
pub mod header;
pub mod parser;
pub mod readable;
pub mod state;
pub mod store;

pub struct Yaz0Stream<UnderlyingStream: ReadableStream<Type = u8>, A: Yaz0StreamReadArgument<Yaz0Parser<UnderlyingStream>>> {
  header: Yaz0Header,
  stream: Yaz0Parser<UnderlyingStream>,
  state: Yaz0State,
  store: A::StoreType,
}

impl<S: ReadableStream<Type = u8>, St: Yaz0StreamReadArgument<Yaz0Parser<S>>> ReadableStream for Yaz0Stream<S, St> {
  type Type = u8;

  type ReadError = Yaz0Error<S::ReadError>;
  type SkipError = Yaz0Error<S::ReadError>;

  fn len(&self) -> Option<u64> {
    Some(self.header.decompressed_size().into())
  }

  fn offset(&self) -> u64 {
    self.state.offset()
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let read_offset = self.offset();
    let mut buffer = heapless::Vec::<u8, SIZE>::new();

    buffer.extend(self.state.take(buffer.capacity() - buffer.len()));

    while self.offset() < self.header.decompressed_size() as u64 && !buffer.is_full() {
      self.store.store_snapshot(&self.stream, self.state.clone());
      let operation = self.stream.read(CLONED).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(_) => StreamReadError::StreamExhausted(StreamExhaustedError {
          read_length: SIZE as u64,
          read_offset,
          stream_length: self.header.decompressed_size() as u64,
        }),
        StreamReadError::User(u) => StreamReadError::User(Yaz0Error::ParseError(StreamReadError::User(u))),
      })?;

      self.state.feed(operation).map_err(|e| Yaz0Error::MalformedStream(e))?;
      buffer.extend(self.state.take(buffer.capacity() - buffer.len()));
    }

    if !buffer.is_full() {
      Err(StreamExhaustedError {
        read_length: SIZE as u64,
        read_offset,
        stream_length: self.header.decompressed_size() as u64,
      })?
    }

    Ok(reader(&buffer.into_array::<SIZE>().unwrap()).await)
  }

  async fn skip(&mut self, mut read_length: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    let read_offset = self.offset();
    let original_read_length = read_length;

    read_length -= self.state.take(read_length as usize).len() as u64;

    while self.offset() < self.header.decompressed_size() as u64 && read_length != 0 {
      self.store.store_snapshot(&self.stream, self.state.clone());
      let block = self.stream.read(CLONED).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(_) => StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
          seek_point: read_offset + original_read_length,
          stream_length: self.header.decompressed_size() as u64,
        }),
        StreamReadError::User(u) => StreamSkipError::User(Yaz0Error::ParseError(StreamReadError::User(u))),
      })?;

      self.state.feed(block).map_err(|e| Yaz0Error::MalformedStream(e))?;
      read_length -= self.state.take(read_length as usize).len() as u64;
    }

    if read_length != 0 {
      Err(StreamSeekOutOfBoundsError {
        seek_point: read_offset + original_read_length,
        stream_length: self.header.decompressed_size() as u64,
      })?
    }

    Ok(())
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream, St: MaybeSnapshotStore<Yaz0Parser<S>>, Sta: Yaz0StreamReadArgument<Yaz0Parser<S>, StoreType = St>> RestorableStream for Yaz0Stream<S, Sta> {
  type Snapshot = (Yaz0State, St, <Yaz0Parser<S> as RestorableStream>::Snapshot);
  type RestoreError = <Yaz0Parser<S> as RestorableStream>::RestoreError;

  fn snapshot(&self) -> Self::Snapshot {
    (self.state.clone(), self.store.clone(), self.stream.snapshot())
  }

  async fn restore(&mut self, snapshot: Self::Snapshot) -> Result<(), StreamRestoreError<Self::RestoreError>> {
    if snapshot.0.offset() <= self.state.offset() {
      self.stream.restore(snapshot.2).await?;
      self.store = snapshot.1;
      self.state = snapshot.0;
      Ok(())
    } else {
      Err(StreamRestoreError::CannotRestoreForwards)
    }
  }
}

const TAIL_LENGTH: usize = 0x111 * 8;

enum ReencodeData {
  Starting(Block),
  StartingWithSkip(Block, u64),
  With(Block),
}

impl<S: ReadableStream<Type = u8> + RestorableStream + ResizableStream + MutableStream, St: SnapshotStore<Yaz0Parser<S>>, Sta: Yaz0StreamReadArgument<Yaz0Parser<S>, StoreType = St>>
  Yaz0Stream<S, Sta>
{
  // PLAN: add `until` function that returns a boolean `true` to continue, `false` to stop.
  // PRECONDITION: `offset` MUST BE CONTAINED WITHIN THE FIRST BLOCK.
  // PRECONDITION: if `offset` is
  async fn re_encode_slice<'a, const C: usize>(
    &mut self,
    state: &mut Yaz0State,
    data: ReencodeData,
    length: &mut u64,
    mut replacement_data: ReadbackReference<'a, C>,
  ) -> Result<
    (Block, (Option<u8>, Block)),
    Yaz0OverwriteError<
      <Yaz0Parser<S> as ReadableStream>::ReadError,
      <Yaz0Parser<S> as RestorableStream>::RestoreError,
      <Yaz0Parser<S> as MutableStream>::MutateError,
      <Yaz0Parser<S> as ResizableStream>::OverwriteError,
    >,
  > {
    let mut tail_block = (None, Block::empty());
    let (mut current_block, mut offset) = match data {
      ReencodeData::Starting(block) => {
        let len = block.len() as u64;
        (block, len)
      }
      ReencodeData::StartingWithSkip(block, skip) => {
        let len = block.len() as u64;
        (block, skip + len)
      }
      ReencodeData::With(block) => (block, 0),
    };

    while let Some(operation) = self.state.compress(&mut replacement_data) {
      self.state.feed_operation(operation).unwrap();
      current_block.operations.push(operation).unwrap();

      if current_block.is_full() {
        if *length > 0 || offset > 0 {
          self
            .stream
            .mutate(async |data: &mut [Block; 1]| {
              let new_tail = if offset > 0 {
                let (_, _, tail_underflow, new_tail) = data[0].clone().split_at_with_mid(offset, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

                if tail_underflow.is_some() {
                  *length -= 1;
                }

                new_tail
              } else {
                data[0].clone()
              };

              let (consumed, consumed_overflow, tail_underflow, new_tail) = new_tail.split_at_with_pre(*length, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

              if consumed_overflow.is_some() {
                *length -= 1;
              }

              *length -= consumed.len() as u64;

              if !new_tail.is_empty() {
                tail_block = (tail_underflow, new_tail);
              }

              data[0] = current_block.clone();

              Ok::<
                _,
                Yaz0OverwriteError<
                  <Yaz0Parser<S> as ReadableStream>::ReadError,
                  <Yaz0Parser<S> as RestorableStream>::RestoreError,
                  <Yaz0Parser<S> as MutableStream>::MutateError,
                  <Yaz0Parser<S> as ResizableStream>::OverwriteError,
                >,
              >(())
            })
            .await
            .map_err(|e| Yaz0OverwriteError::MutateBlockFailed(e))??;
        } else {
          self.stream.overwrite(0, [current_block.clone()]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        }

        offset = 0;
        current_block = Block::empty();
      }
    }

    if *length > 0 {
      let mut overwrite_count = 0;

      let snapshot = self.stream.snapshot();

      while *length > 0 {
        self
          .stream
          .read(async |data: &[Block; 1]| {
            let new_tail = if offset > 0 {
              let (_, _, tail_underflow, new_tail) = data[0].clone().split_at_with_mid(offset, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

              if tail_underflow.is_some() {
                *length -= 1;
              }

              new_tail
            } else {
              data[0].clone()
            };

            let (consumed, consumed_overflow, tail_underflow, new_tail) = new_tail.split_at_with_pre(*length, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

            if consumed_overflow.is_some() {
              *length -= 1;
            }

            *length -= consumed.len() as u64;

            if !new_tail.is_empty() {
              tail_block = (tail_underflow, new_tail);
            }

            Ok::<
              _,
              Yaz0OverwriteError<
                <Yaz0Parser<S> as ReadableStream>::ReadError,
                <Yaz0Parser<S> as RestorableStream>::RestoreError,
                <Yaz0Parser<S> as MutableStream>::MutateError,
                <Yaz0Parser<S> as ResizableStream>::OverwriteError,
              >,
            >(())
          })
          .await
          .map_err(|e| Yaz0OverwriteError::ReadBlockFailed(e))??;

        offset = 0;
        overwrite_count += 1;
      }

      self.stream.restore(snapshot).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;
      self.stream.overwrite(overwrite_count, []).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
    }

    Ok((current_block, tail_block))
  }
}

impl<S: ReadableStream<Type = u8> + RestorableStream + ResizableStream + MutableStream, St: SnapshotStore<Yaz0Parser<S>>, Sta: Yaz0StreamReadArgument<Yaz0Parser<S>, StoreType = St>> ResizableStream
  for Yaz0Stream<S, Sta>
where
  <S as RestorableStream>::Snapshot: Debug,
{
  type OverwriteError = Yaz0OverwriteError<
    <Yaz0Parser<S> as ReadableStream>::ReadError,
    <Yaz0Parser<S> as RestorableStream>::RestoreError,
    <Yaz0Parser<S> as MutableStream>::MutateError,
    <Yaz0Parser<S> as ResizableStream>::OverwriteError,
  >;

  async fn overwrite<const SIZE: usize>(&mut self, mut length: u64, data: [Self::Type; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    let current_offset = self.offset();
    let current_block = self.state.current_block().clone();

    if let Some(snapshot) = self.store.snapshot().cloned() {
      self.stream.restore(snapshot).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;
      self.state = self.store.state();
    } else {
      assert!(current_offset == 0);
    };

    let block_offset = current_offset - self.offset();

    let (starting_block, starting_overflow, _, _) = current_block.split_at_with_pre(block_offset, &mut self.state).unwrap();

    let mut original_state = self.state.clone();

    let reencode_data = if let Some(overflow) = starting_overflow {
      let skip_len = starting_block.len();
      self.stream.overwrite(0, [starting_block]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
      ReencodeData::StartingWithSkip(Block::of(Operation::lit(overflow)), skip_len as u64)
    } else {
      ReencodeData::Starting(starting_block)
    };

    println!("state: {:?}", self.state.readback().collect::<heapless::Vec<u8, 0x1000>>());

    let (current_block, tail) = self.re_encode_slice(&mut original_state, reencode_data, &mut length, ReadbackReference::of(&data)).await?;
    let tail_len = tail.1.len() as u64 + (if tail.0.is_some() { 1 } else { 0 });

    let mut repair_bytes = 0;
    let mut bytes_seeked = tail_len;
    let pre_read = self.stream.snapshot();

    let mut fork_original_state = original_state.clone();

    if let Some(overflow) = tail.0 {
      fork_original_state.feed(Block::of(Operation::lit(overflow))).unwrap();
    }

    fork_original_state.feed(tail.1).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

    while bytes_seeked < 4096 {
      let block: Block = self.stream.read(CLONED).await.map_err(|e| Yaz0OverwriteError::ReadBlockFailed(e))?;
      let (head, head_overflow, _, _) = block
        .split_at_with_pre(4096 - bytes_seeked, &mut fork_original_state)
        .map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

      for operation in head.operations {
        bytes_seeked += operation.len() as u64;

        if let Operation::LongReadback { offset, .. } | Operation::ShortReadback { offset, .. } = operation {
          if (offset.get() as u64).saturating_sub(bytes_seeked) > 0 {
            repair_bytes = bytes_seeked;
          }
        }
      }

      if head_overflow.is_some() {
        bytes_seeked += 1;
      }
    }

    self.stream.restore(pre_read).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;

    let repair = fork_original_state.readback().slice(0..(repair_bytes - tail_len) as usize).unwrap();

    let repair_len = repair.len();
    println!("state: {:?}", self.state.readback().collect::<heapless::Vec<u8, 0x1000>>());

    let (mut block, tail) = self.re_encode_slice(&mut original_state, ReencodeData::With(current_block), &mut (repair_len as u64), repair).await?;

    if let Some(byte) = tail.0 {
      // we KNOW block is not full
      block.operations.push(Operation::Literal(byte)).unwrap();
    }

    let mut tail = tail.1;

    dbg!(block.clone(), tail.clone());
    println!("state: {:?}", self.state.readback().collect::<heapless::Vec<u8, 0x1000>>());

    // self.state.feed(block.clone()).unwrap();
    // self.state.feed(tail.clone()).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

    loop {
      inflate_pair([&mut block, &mut tail], &self.state).unwrap();

      block = if !block.is_full() {
        if self.stream.remaining_decoded_bytes() == 0 {
          self.stream.overwrite(0, [block]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
          return Ok(());
        }

        block
      } else if !tail.is_full() {
        self.stream.overwrite(0, [block]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        tail
      } else {
        self.stream.overwrite(0, [block]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        self.stream.overwrite(0, [tail]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;

        return Ok(());
      };

      if block.is_empty() {
        return Ok(());
      }

      tail = self.stream.read(CLONED).await.map_err(|e| Yaz0OverwriteError::ReadBlockFailed(e))?;
      self.state.feed(tail.clone()).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
    }
  }
}

// RewindableStream NOT FEASIBLE :(
// SeekableStream NOT FEASIBLE :(
// MutableStream FEASIBLE :) GIVEN Substream: RestorableStream + ResizableStream + MutableStream
// ResizableStream FEASIBLE :) GIVEN Substream: RestorableStream + ResizableStream + MutableStream
// RestorableStream FEASIBLE :) GIVEN Substream: RestorableStream
