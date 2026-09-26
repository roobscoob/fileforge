use core::u32;

use fileforge::control_flow::ControlFlow;
use fileforge::stream::{
  error::{
    stream_exhausted::StreamExhaustedError, stream_mutate::StreamMutateError, stream_overwrite::StreamOverwriteError, stream_read::StreamReadError, stream_restore::StreamRestoreError,
    stream_seek_out_of_bounds::StreamSeekOutOfBoundsError, stream_skip::StreamSkipError,
  },
  MutableStream, ReadableStream, ResizableStream, RestorableStream, StaticPartitionableStream, CLONED,
};

use crate::sead::yaz0::{
  error::{mutate::Yaz0MutateError, overwrite::Yaz0OverwriteError, Yaz0Error},
  header::YAZ0_HEADER_SIZE,
  parser::{
    block_inflate_pair::inflate_pair,
    data::{Block, Operation},
    Yaz0Parser,
  },
  readable::{HeaderView, MutHeaderView, Yaz0StreamReadArgument},
  state::{reference::ReadbackReference, Yaz0State},
  store::{MaybeSnapshotStore, SnapshotStore},
};

pub mod error;
pub mod header;
pub mod parser;
pub mod readable;
pub mod state;
pub mod store;

pub struct Yaz0Stream<'pool, OriginalStream: ReadableStream<Type = u8>, A: Yaz0StreamReadArgument<'pool, OriginalStream>> {
  header: A::HeaderView,
  stream: Yaz0Parser<<A::HeaderView as HeaderView<'pool, OriginalStream>>::OtherStream>,
  state: Yaz0State,
  store: A::StoreType,
}

impl<'pool, S: ReadableStream<Type = u8>, St: Yaz0StreamReadArgument<'pool, S>> ReadableStream for Yaz0Stream<'pool, S, St> {
  type Type = u8;

  type ReadError = Yaz0Error<<<St::HeaderView as HeaderView<'pool, S>>::OtherStream as ReadableStream>::ReadError>;
  type SkipError = Yaz0Error<<<St::HeaderView as HeaderView<'pool, S>>::OtherStream as ReadableStream>::ReadError>;

  fn len(&self) -> Option<u64> {
    Some(self.header.value().decompressed_size().into())
  }

  fn offset(&self) -> u64 {
    self.state.offset()
  }

  async fn read<const SIZE: usize, V>(&mut self, reader: impl AsyncFnOnce(&[Self::Type; SIZE]) -> V) -> Result<V, StreamReadError<Self::ReadError>> {
    let read_offset = self.offset();
    let mut buffer = heapless::Vec::<u8, SIZE>::new();

    buffer.extend(self.state.take(buffer.capacity() - buffer.len()));

    while self.offset() < self.header.value().decompressed_size() as u64 && !buffer.is_full() {
      self.store.store_snapshot(&self.stream, self.state.clone());
      let operation = self.stream.read(CLONED).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(_) => StreamReadError::StreamExhausted(StreamExhaustedError {
          read_length: SIZE as u64,
          read_offset,
          stream_length: self.header.value().decompressed_size() as u64,
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
        stream_length: self.header.value().decompressed_size() as u64,
      })?
    }

    Ok(reader(&buffer.into_array::<SIZE>().unwrap()).await)
  }

  async fn skip(&mut self, mut read_length: u64) -> Result<(), StreamSkipError<Self::SkipError>> {
    let read_offset = self.offset();
    let original_read_length = read_length;

    read_length -= self.state.take(read_length as usize).len() as u64;

    while self.offset() < self.header.value().decompressed_size() as u64 && read_length != 0 {
      self.store.store_snapshot(&self.stream, self.state.clone());
      let block = self.stream.read(CLONED).await.map_err(|e| match e {
        StreamReadError::StreamExhausted(_) => StreamSkipError::OutOfBounds(StreamSeekOutOfBoundsError {
          seek_point: read_offset + original_read_length,
          stream_length: self.header.value().decompressed_size() as u64,
        }),
        StreamReadError::User(u) => StreamSkipError::User(Yaz0Error::ParseError(StreamReadError::User(u))),
      })?;

      self.state.feed(block).map_err(|e| Yaz0Error::MalformedStream(e))?;
      read_length -= self.state.take(read_length as usize).len() as u64;
    }

    if read_length != 0 {
      Err(StreamSeekOutOfBoundsError {
        seek_point: read_offset + original_read_length,
        stream_length: self.header.value().decompressed_size() as u64,
      })?
    }

    Ok(())
  }
}

impl<'pool, S: ReadableStream<Type = u8>, Sta: Yaz0StreamReadArgument<'pool, S>> RestorableStream for Yaz0Stream<'pool, S, Sta>
where
  <Sta::HeaderView as HeaderView<'pool, S>>::OtherStream: RestorableStream,
{
  type Snapshot = (Yaz0State, Sta::StoreType, <Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as RestorableStream>::Snapshot);
  type RestoreError = <Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as RestorableStream>::RestoreError;

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

enum ReencodeData {
  Starting(Block),
  StartingWithSkip(Block, u64),
  With(Block),
}

impl<'pool, S: ReadableStream<Type = u8> + StaticPartitionableStream<YAZ0_HEADER_SIZE>, Sta: Yaz0StreamReadArgument<'pool, S>> Yaz0Stream<'pool, S, Sta>
where
  <Sta::HeaderView as HeaderView<'pool, S>>::OtherStream: RestorableStream + ResizableStream + MutableStream,
  <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft: MutableStream<Type = u8> + RestorableStream,
  Sta::HeaderView: MutHeaderView<'pool, S, <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft>,
  Sta::StoreType: SnapshotStore<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream>,
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
  ) -> Result<(Block, (Option<u8>, Block)), <Self as ResizableStream>::OverwriteError> {
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

      // the block we start from can already be full; flush it before pushing.
      if current_block.is_full() {
        if *length > 0 || offset > 0 {
          self
            .stream
            .mutate(async |data: &mut [Block; 1]| {
              let (new_tail, leading) = if offset > 0 {
                let (_, _, leading, new_tail) = data[0].clone().split_at_with_mid(offset, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
                (new_tail, leading)
              } else {
                (data[0].clone(), None)
              };

              // a byte that didn't fit in `new_tail` is the first byte after the edit point. If bytes
              // are being replaced it's the first of them: consume it, and feed it into the original
              // history. Otherwise keep it: it leads the tail.
              let leading = match leading {
                Some(byte) if *length > 0 => {
                  *length -= 1;
                  state.feed(Block::of(Operation::lit(byte))).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
                  None
                }
                leading => leading,
              };

              let (consumed, consumed_overflow, tail_underflow, new_tail) = new_tail.split_at_with_pre(*length, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

              if consumed_overflow.is_some() {
                *length -= 1;
              }

              *length -= consumed.len() as u64;

              if !new_tail.is_empty() || leading.is_some() {
                tail_block = (leading.or(tail_underflow), new_tail);
              }

              data[0] = current_block.clone();

              Ok::<_, <Self as ResizableStream>::OverwriteError>(())
            })
            .await
            .map_err(|e| Yaz0OverwriteError::MutateBlockFailed(e))??;
        } else {
          self.stream.overwrite(0, [current_block.clone()]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        }

        offset = 0;
        current_block = Block::empty();
      }

      current_block.operations.push(operation).unwrap();

      if current_block.is_full() {
        if *length > 0 || offset > 0 {
          self
            .stream
            .mutate(async |data: &mut [Block; 1]| {
              let (new_tail, leading) = if offset > 0 {
                let (_, _, leading, new_tail) = data[0].clone().split_at_with_mid(offset, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
                (new_tail, leading)
              } else {
                (data[0].clone(), None)
              };

              // a byte that didn't fit in `new_tail` is the first byte after the edit point. If bytes
              // are being replaced it's the first of them: consume it, and feed it into the original
              // history. Otherwise keep it: it leads the tail.
              let leading = match leading {
                Some(byte) if *length > 0 => {
                  *length -= 1;
                  state.feed(Block::of(Operation::lit(byte))).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
                  None
                }
                leading => leading,
              };

              let (consumed, consumed_overflow, tail_underflow, new_tail) = new_tail.split_at_with_pre(*length, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

              if consumed_overflow.is_some() {
                *length -= 1;
              }

              *length -= consumed.len() as u64;

              if !new_tail.is_empty() || leading.is_some() {
                tail_block = (leading.or(tail_underflow), new_tail);
              }

              data[0] = current_block.clone();

              Ok::<_, <Self as ResizableStream>::OverwriteError>(())
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

    // also consume the original block when the edit starts partway into it
    if *length > 0 || offset > 0 {
      let mut overwrite_count = 0;

      let snapshot = self.stream.snapshot();

      while *length > 0 || offset > 0 {
        self
          .stream
          .read(async |data: &[Block; 1]| {
            let (new_tail, leading) = if offset > 0 {
              let (_, _, leading, new_tail) = data[0].clone().split_at_with_mid(offset, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
              (new_tail, leading)
            } else {
              (data[0].clone(), None)
            };

            // a byte that didn't fit in `new_tail` is the first byte after the edit point. If bytes
            // are being replaced it's the first of them: consume it, and feed it into the original
            // history. Otherwise keep it: it leads the tail.
            let leading = match leading {
              Some(byte) if *length > 0 => {
                *length -= 1;
                state.feed(Block::of(Operation::lit(byte))).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
                None
              }
              leading => leading,
            };

            let (consumed, consumed_overflow, tail_underflow, new_tail) = new_tail.split_at_with_pre(*length, state).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

            if consumed_overflow.is_some() {
              *length -= 1;
            }

            *length -= consumed.len() as u64;

            if !new_tail.is_empty() || leading.is_some() {
              tail_block = (leading.or(tail_underflow), new_tail);
            }

            Ok::<_, <Self as ResizableStream>::OverwriteError>(())
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

  /// Makes the edit, and returns where the block containing the edit point starts: nothing before it
  /// changes, so the caller can decode forward from there to land after the new data.
  async fn overwrite_in_place<const SIZE: usize>(
    &mut self,
    mut length: u64,
    data: [u8; SIZE],
  ) -> Result<
    (<Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as RestorableStream>::Snapshot, Yaz0State),
    StreamOverwriteError<<Self as ResizableStream>::OverwriteError>,
  > {
    let current_offset = self.offset();
    let current_block = self.state.current_block().clone();

    if let Some(snapshot) = self.store.snapshot().cloned() {
      self.stream.restore(snapshot).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;
      self.state = self.store.state();
    } else {
      assert!(current_offset == 0);
    };

    let start = (self.stream.snapshot(), self.state.clone());

    let uncompressed_size = self
      .header
      .value()
      .decompressed_size()
      .saturating_sub(length.try_into().unwrap_or(u32::MAX))
      .checked_add(data.len().try_into().map_err(|_| Yaz0OverwriteError::TooMuchData)?)
      .ok_or(Yaz0OverwriteError::TooMuchData)?;

    self
      .header
      .mutate()
      .await
      .map_err(|e| Yaz0OverwriteError::MutateHeaderError(e))?
      .with_uncompressed_size(uncompressed_size)
      .await
      .map_err(|e| Yaz0OverwriteError::MutateHeaderFieldError(e))?;

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

    let (current_block, tail) = self.re_encode_slice(&mut original_state, reencode_data, &mut length, ReadbackReference::of(&data)).await?;
    let tail_len = tail.1.len() as u64 + (if tail.0.is_some() { 1 } else { 0 });

    let mut repair_bytes = 0;
    let mut bytes_seeked = tail_len;
    let pre_read = self.stream.snapshot();

    let mut fork_original_state = original_state.clone();

    if let Some(overflow) = tail.0 {
      fork_original_state.feed(Block::of(Operation::lit(overflow))).unwrap();
    }

    fork_original_state.feed(tail.1.clone()).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

    // the repair consumes the original blocks after the tail, decoding them against
    // `original_state`, so it needs the tail's bytes too.
    if let Some(overflow) = tail.0 {
      original_state.feed(Block::of(Operation::lit(overflow))).unwrap();
    }
    original_state.feed(tail.1.clone()).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

    // stop at the end of the data, not only after 4 KB
    while bytes_seeked < 4096 && self.stream.remaining_decoded_bytes() > 0 {
      let block: Block = self.stream.read(CLONED).await.map_err(|e| Yaz0OverwriteError::ReadBlockFailed(e))?;

      // a readback crosses the edit if it reaches back past the tail's start from where it
      // *starts*; the repair must then cover it, up to the 4096-byte limit (past the limit, the
      // rest of it can't reach back that far). Check the whole block before splitting it at the
      // limit: the split turns the head of a readback cut 1 or 2 bytes in into literals.
      let mut starts_at = bytes_seeked;
      for operation in block.operations.iter() {
        if starts_at >= 4096 {
          break;
        }

        if let Operation::LongReadback { offset, .. } | Operation::ShortReadback { offset, .. } = operation {
          if offset.get() as u64 > starts_at {
            repair_bytes = (starts_at + operation.len() as u64).min(4096);
          }
        }

        starts_at += operation.len() as u64;
      }

      let (head, head_overflow, _, _) = block
        .split_at_with_pre(4096 - bytes_seeked, &mut fork_original_state)
        .map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

      bytes_seeked += head.len() as u64;

      if head_overflow.is_some() {
        bytes_seeked += 1;
      }
    }

    self.stream.restore(pre_read).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;

    // the repair re-encodes the tail plus everything up to the last crossing readback. The
    // history ends `bytes_seeked` bytes after the tail's start, so find the window from the end.
    let repair_bytes = repair_bytes.max(tail_len);
    let history = fork_original_state.readback();
    let window_start = history.len() - bytes_seeked as usize;
    let repair = history.slice(window_start..window_start + repair_bytes as usize).unwrap();

    let mut repair_len = repair.len() as u64 - tail_len;

    let (mut block, tail) = self.re_encode_slice(&mut original_state, ReencodeData::With(current_block), &mut repair_len, repair).await?;

    if let Some(byte) = tail.0 {
      // we KNOW block is not full
      block.operations.push(Operation::Literal(byte)).unwrap();

      // the byte is new output now, so the new history needs it too, or everything decoded
      // against it afterwards (the tail, and `inflate_pair`'s literals) is shifted by one.
      self.state.feed_operation(Operation::Literal(byte)).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
    }

    let mut tail = tail.1;
    let mut overwrite = false;

    loop {
      self.state.feed(tail.clone()).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;

      inflate_pair([&mut block, &mut tail], &self.state).unwrap();

      block = if !block.is_full() {
        // the original block at the stream position has been absorbed into `block`.
        // Remove it, or the next read would read (and merge) it again.
        if overwrite {
          self.stream.overwrite(1, []).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        }

        // nothing left to write means nothing to write: an empty block is still a header byte.
        if self.stream.remaining_decoded_bytes() == 0 {
          if !block.is_empty() {
            self.stream.overwrite(0, [block]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
          }
          return Ok(start);
        }

        block
      } else if !tail.is_full() {
        self
          .stream
          .overwrite(if overwrite { 1 } else { 0 }, [block])
          .await
          .map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;

        tail
      } else {
        self
          .stream
          .overwrite(if overwrite { 1 } else { 0 }, [block])
          .await
          .map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        self.stream.overwrite(0, [tail]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;

        return Ok(start);
      };

      if block.is_empty() {
        return Ok(start);
      }

      // leftovers after writing a full block, but no original data left to merge them with:
      // write them as the final block and stop, instead of reading past the end.
      if self.stream.remaining_decoded_bytes() == 0 {
        self.stream.overwrite(0, [block]).await.map_err(|e| Yaz0OverwriteError::OverwriteBlockFailed(e))?;
        return Ok(start);
      }

      let back = self.stream.snapshot();
      tail = self.stream.read(CLONED).await.map_err(|e| Yaz0OverwriteError::ReadBlockFailed(e))?;
      self.stream.restore(back).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;

      overwrite = true;
    }
  }
}

impl<'pool, S: ReadableStream<Type = u8> + StaticPartitionableStream<YAZ0_HEADER_SIZE>, Sta: Yaz0StreamReadArgument<'pool, S>> ResizableStream for Yaz0Stream<'pool, S, Sta>
where
  <Sta::HeaderView as HeaderView<'pool, S>>::OtherStream: RestorableStream + ResizableStream + MutableStream,
  <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft: MutableStream<Type = u8> + RestorableStream,
  Sta::HeaderView: MutHeaderView<'pool, S, <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft>,
  Sta::StoreType: SnapshotStore<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream>,
{
  type OverwriteError = Yaz0OverwriteError<
    'pool,
    <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft,
    <Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as ReadableStream>::ReadError,
    <Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as RestorableStream>::RestoreError,
    <Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as MutableStream>::MutateError,
    <Yaz0Parser<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream> as ResizableStream>::OverwriteError,
  >;

  async fn overwrite<const SIZE: usize>(&mut self, length: u64, data: [Self::Type; SIZE]) -> Result<(), StreamOverwriteError<Self::OverwriteError>> {
    let current_offset = self.offset();
    let size = self.header.value().decompressed_size() as u64;

    // Check the bounds before changing anything, as `ProviderStream` does.
    if length > size - current_offset {
      return Err(StreamOverwriteError::StreamExhausted(StreamExhaustedError {
        read_length: length,
        read_offset: current_offset,
        stream_length: size,
      }));
    }

    let (snapshot, state) = self.overwrite_in_place(length, data).await?;

    // Land just after the new data, as `ProviderStream` does: go back to the start of the edited
    // block and decode forward, which leaves the history, the position and the store as a read would.
    self.stream.restore(snapshot).await.map_err(|e| Yaz0OverwriteError::RestoreFailed(e))?;
    self.state = state;

    let mut remaining = current_offset + SIZE as u64 - self.offset();
    remaining -= self.state.take(remaining as usize).len() as u64;

    while remaining > 0 {
      self.store.store_snapshot(&self.stream, self.state.clone());
      let block = self.stream.read(CLONED).await.map_err(|e| Yaz0OverwriteError::ReadBlockFailed(e))?;
      self.state.feed(block).map_err(|e| Yaz0OverwriteError::MalformedStream(e))?;
      remaining -= self.state.take(remaining as usize).len() as u64;
    }

    Ok(())
  }
}

impl<'pool, S: ReadableStream<Type = u8> + StaticPartitionableStream<YAZ0_HEADER_SIZE>, Sta: Yaz0StreamReadArgument<'pool, S>> MutableStream for Yaz0Stream<'pool, S, Sta>
where
  <Sta::HeaderView as HeaderView<'pool, S>>::OtherStream: RestorableStream + ResizableStream + MutableStream,
  <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft: MutableStream<Type = u8> + RestorableStream,
  Sta::HeaderView: MutHeaderView<'pool, S, <S as StaticPartitionableStream<YAZ0_HEADER_SIZE>>::PartitionLeft>,
  Sta::StoreType: SnapshotStore<<Sta::HeaderView as HeaderView<'pool, S>>::OtherStream>,
{
  type MutateError = Yaz0MutateError<Self::ReadError, <Self as RestorableStream>::RestoreError, <Self as ResizableStream>::OverwriteError>;

  /// Reads the bytes, returns to them, and overwrites them with the changed ones: a same-size
  /// [`overwrite`](ResizableStream::overwrite), which re-encodes only locally.
  async fn mutate<const SIZE: usize, V: ControlFlow>(&mut self, mutator: impl AsyncFnOnce(&mut [u8; SIZE]) -> V) -> Result<V, StreamMutateError<Self::MutateError>> {
    let snapshot = self.snapshot();

    let mut data = match self.read(async |data: &[u8; SIZE]| *data).await {
      Ok(data) => data,
      Err(StreamReadError::StreamExhausted(exhausted)) => return Err(StreamMutateError::StreamExhausted(exhausted)),
      Err(error) => return Err(StreamMutateError::User(Yaz0MutateError::Read(error))),
    };

    self.restore(snapshot).await.map_err(|error| StreamMutateError::User(Yaz0MutateError::Restore(error)))?;

    let value = mutator(&mut data).await;

    self.overwrite(SIZE as u64, data).await.map_err(|error| StreamMutateError::User(Yaz0MutateError::Overwrite(error)))?;

    Ok(value)
  }
}

// RewindableStream NOT FEASIBLE :(
// SeekableStream NOT FEASIBLE :(
// ResizableStream FEASIBLE :) GIVEN Substream: RestorableStream + ResizableStream + MutableStream
// RestorableStream FEASIBLE :) GIVEN Substream: RestorableStream
