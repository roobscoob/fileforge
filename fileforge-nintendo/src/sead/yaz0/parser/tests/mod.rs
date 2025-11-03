use std::{
  num::NonZeroU16,
  sync::atomic::{AtomicBool, Ordering},
};

use crate::sead::yaz0::parser::{
  self,
  data::{Block, Operation},
  Yaz0Parser,
};
use fileforge_lib::{
  control_flow::Continue,
  provider::hint::ReadHint,
  stream::{builtin::provider::ProviderStream, MutableStream, ReadableStream, ResizableStream, RestorableStream, CLONED},
};

/// --------- Small helpers to build fixtures ---------

fn lit(b: u8) -> Operation {
  Operation::Literal(b)
}

fn srb(offset: u16, len: u16) -> Operation {
  Operation::ShortReadback {
    offset: NonZeroU16::new(offset).unwrap(),
    length: NonZeroU16::new(len).unwrap(),
  }
}

fn lrb(offset: u16, len: u16) -> Operation {
  Operation::LongReadback {
    offset: NonZeroU16::new(offset).unwrap(),
    length: NonZeroU16::new(len).unwrap(),
  }
}

// Encode a single op as Yaz0 bytes.
fn enc_op(op: &Operation) -> Vec<u8> {
  match *op {
    Operation::Literal(b) => vec![b],
    Operation::ShortReadback { offset, length } => {
      // 2-byte: NR RR   with N=(len-2), RRR=(off-1)
      let n = (length.get() - 2) as u16; // 1..=0xF
      let r = (offset.get() - 1) as u16;
      let b0 = ((n & 0xF) << 4) as u8 | ((r >> 8) as u8 & 0x0F);
      let b1 = (r & 0xFF) as u8;
      vec![b0, b1]
    }
    Operation::LongReadback { offset, length } => {
      // 3-byte: 0R RR NN with NN=(len-0x12)
      let r = (offset.get() - 1) as u16;
      let n = (length.get() - 0x12) as u8;
      let b0 = (r >> 8) as u8; // high nibble 0 satisfies (b0 & 0xF0) == 0
      let b1 = (r & 0xFF) as u8;
      vec![b0, b1, n]
    }
  }
}

fn enc_block(block: &Block) -> Vec<u8> {
  // All non-final blocks we build in tests will have 8 ops.
  let mut out = vec![block.compute_header()];
  for op in &block.operations {
    out.extend_from_slice(&enc_op(op));
  }
  out
}

fn enc_blocks(blocks: &[Block]) -> Vec<u8> {
  let mut out = Vec::new();
  for b in blocks {
    out.extend_from_slice(&enc_block(b));
  }
  out
}

fn decoded_len_blocks(blocks: &[Block]) -> u32 {
  blocks.iter().flat_map(|b| b.operations.iter()).map(|v| v.len() as u32).sum()
}

fn prov(bytes: Vec<u8>) -> ProviderStream<Vec<u8>> {
  ProviderStream::new(bytes, ReadHint::new())
}

/// ------------------------------
/// Tests (happy-path only)
/// ------------------------------

#[tokio::test]
async fn parse_full_block_of_8_literals() {
  // Non-final block must have exactly 8 ops.
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(1), lit(2), lit(3), lit(4), lit(5), lit(6), lit(7), lit(8)]).unwrap(),
  };

  // Make it the only block (also final), still valid.
  let bytes = enc_blocks(&[b0.clone()]);
  let decoded_len = decoded_len_blocks(&[b0.clone()]);
  let mut parser = Yaz0Parser::new(prov(bytes), decoded_len);

  // read(SINGLE) → Block
  let blk = parser.read(CLONED).await.unwrap();
  assert_eq!(blk.operations, b0.operations);
}

#[tokio::test]
async fn parse_two_blocks_first_has_8_ops_last_shorter() {
  // First (non-final) block: exactly 8 ops
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(0x11), srb(0x012, 3), lrb(0x020, 0x40), lit(0x22), srb(0x123, 5), lit(0x33), lit(0x44), srb(0x005, 5)]).unwrap(),
  };

  // Final block: fewer than 8 ops is allowed
  let b1 = Block {
    operations: heapless::Vec::from_slice(&[lit(0xAA), srb(0x2A3, 6), lit(0xBB)]).unwrap(),
  };

  let bytes = enc_blocks(&[b0.clone(), b1.clone()]);
  let decoded_len = decoded_len_blocks(&[b0.clone(), b1.clone()]);
  let mut parser = Yaz0Parser::new(prov(bytes), decoded_len);

  let first = parser.read(CLONED).await.unwrap();
  assert_eq!(first.operations, b0.operations);

  let second = parser.read(CLONED).await.unwrap();
  assert_eq!(second.operations, b1.operations);
}

#[tokio::test]
async fn skip_one_block_then_read_next() {
  // First block must be 8 ops; second is final and can be shorter.
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(9), lit(8), lit(7), lit(6), lit(5), lit(4), lit(3), lit(2)]).unwrap(),
  };
  let b1 = Block {
    operations: heapless::Vec::from_slice(&[srb(0x010, 3), lit(0xCC), lrb(0x020, 0x30)]).unwrap(),
  };

  let bytes = enc_blocks(&[b0.clone(), b1.clone()]);
  let decoded_len = decoded_len_blocks(&[b0.clone(), b1.clone()]);
  let mut parser = Yaz0Parser::new(prov(bytes), decoded_len);

  parser.skip(1).await.unwrap(); // skip exactly one block
  let blk = parser.read(CLONED).await.unwrap();
  assert_eq!(blk.operations, b1.operations);
}

#[tokio::test]
async fn roundtrip_header_and_encoding() {
  // First block (non-final): exactly 8 ops.
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(0x10), srb(0x012, 3), lit(0x20), lrb(0x034, 0x20), lit(0x30), srb(0x056, 5), lit(0x40), lit(0x50)]).unwrap(),
  };
  // Final block: 4 ops.
  let b1 = Block {
    operations: heapless::Vec::from_slice(&[lit(0xDE), lit(0xAD), srb(0x2A3, 6), lit(0xBE)]).unwrap(),
  };

  let bytes = enc_blocks(&[b0.clone(), b1.clone()]);
  let decoded_len = decoded_len_blocks(&[b0.clone(), b1.clone()]);

  let mut parser = Yaz0Parser::new(prov(bytes.clone()), decoded_len);

  // Read both blocks one-by-one
  let r0 = parser.read(CLONED).await.unwrap();
  let r1 = parser.read(CLONED).await.unwrap();

  assert_eq!(r0.operations, b0.operations);
  assert_eq!(r1.operations, b1.operations);

  // Re-encode parsed data and compare bytes
  let re = enc_blocks(&[r0, r1]);
  assert_eq!(re, bytes);
}

#[tokio::test]
async fn mutate_in_place_then_verify_via_followup_mutate() {
  let before = Block {
    operations: heapless::Vec::from_slice(&[lit(0x10), lit(0x20), lit(0x30), lit(0x40), lit(0x50), lit(0x60), lit(0x70), lit(0x80)]).unwrap(),
  };
  let bytes = enc_blocks(&[before.clone()]);
  let mut parser = Yaz0Parser::new(prov(bytes), decoded_len_blocks(&[before.clone()]));

  // Desired post-mutation version of the first block.
  let mut after = before.clone();
  after.operations[1] = srb(0x2A3, 6);

  let snapshot = parser.snapshot();

  // 1) Perform the mutation.
  parser
    .mutate::<1, _>(async move |blocks| {
      blocks[0].operations[1] = srb(0x2A3, 6);
    })
    .await
    .unwrap();

  parser.restore(snapshot).await.unwrap();

  // 2) Verify by inspecting the block provided to a *second* mutate.
  let verified = AtomicBool::new(false);
  let expected = after.clone();
  parser
    .mutate::<1, _>(async |blocks| {
      if blocks[0].operations == expected.operations {
        verified.store(true, Ordering::SeqCst);
      }
    })
    .await
    .unwrap();

  assert!(verified.load(Ordering::SeqCst), "follow-up mutate saw the expected mutated block");
}

/// Mutate multiple op kinds; keep 8 ops; verify via follow-up mutate.
#[tokio::test]
async fn mutate_multiple_ops_and_verify() {
  let before = Block {
    operations: heapless::Vec::from_slice(&[lit(1), srb(0x010, 3), lit(2), lrb(0x020, 0x40), lit(3), srb(0x111, 5), lit(4), lit(5)]).unwrap(),
  };
  let bytes = enc_blocks(&[before.clone()]);
  let mut parser = Yaz0Parser::new(prov(bytes), decoded_len_blocks(&[before.clone()]));

  let mut after = before.clone();
  after.operations[0] = srb(0x222, 6);
  after.operations[3] = lit(0xAB);
  after.operations[6] = srb(0x333, 4);

  let snapshot = parser.snapshot();

  // perform mutate
  parser
    .mutate::<1, _>(async |blocks| {
      let b = &mut blocks[0];
      b.operations[0] = srb(0x222, 6);
      b.operations[3] = lit(0xAB);
      b.operations[6] = srb(0x333, 4);
    })
    .await
    .unwrap();

  parser.restore(snapshot).await.unwrap();

  // verify via follow-up mutate
  let verified = AtomicBool::new(false);
  let expected = after.clone();
  parser
    .mutate::<1, _>(async |blocks| {
      if blocks[0].operations == expected.operations {
        verified.store(true, Ordering::SeqCst);
      }
    })
    .await
    .unwrap();

  assert!(verified.load(Ordering::SeqCst));
}

/// Overwrite length=0 with two blocks (pure append on empty).
/// Then read both blocks to confirm exact values.
#[tokio::test]
async fn overwrite_appends_on_empty_and_read_back() {
  let nb0 = Block {
    operations: heapless::Vec::from_slice(&[srb(0x010, 3), srb(0x020, 4), lit(0xAB), lit(0xCD), lit(0xEF), lit(0x01), lit(0x02), lit(0x03)]).unwrap(),
  };
  let nb1 = Block {
    operations: heapless::Vec::from_slice(&[lit(0xDE), srb(0x2A3, 6), lit(0xAD)]).unwrap(),
  };

  // start empty stream
  let mut parser = Yaz0Parser::new(prov(vec![]), 0);

  let snapshot = parser.snapshot();

  parser.overwrite::<2>(0, [nb0.clone(), nb1.clone()]).await.unwrap();

  parser.restore(snapshot).await.unwrap();

  // read back two blocks
  let r0 = parser.read(CLONED).await.unwrap();
  let r1 = parser.read(CLONED).await.unwrap();
  assert_eq!(r0.operations.len(), 8);
  assert_eq!(r0.operations, nb0.operations);
  assert!(r1.operations.len() < 8); // final may be shorter
  assert_eq!(r1.operations, nb1.operations);
}

/// Overwrite replaces first block and appends a second; verify via follow-up mutates.
#[tokio::test]
async fn overwrite_replace_and_append_then_verify_via_mutate() {
  // Initial: b0 (8 ops)
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(0x11), lit(0x22), lit(0x33), lit(0x44), lit(0x55), lit(0x66), srb(0x100, 3), lit(0x77)]).unwrap(),
  };
  let mut parser = Yaz0Parser::new(prov(enc_blocks(&[b0.clone()])), decoded_len_blocks(&[b0.clone()]));

  // New: nb0 replaces b0; nb1 gets appended
  let nb0 = Block {
    operations: heapless::Vec::from_slice(&[srb(0x010, 3), lit(0x99), lit(0x98), lit(0x97), lit(0x96), lit(0x95), lit(0x94), lit(0x93)]).unwrap(),
  };
  let nb1 = Block {
    operations: heapless::Vec::from_slice(&[lrb(0x020, 0x40), lit(0xCC), srb(0x005, 5)]).unwrap(),
  };

  let snapshot = parser.snapshot();

  // length=1 → replace one block; extra data appends one more
  parser.overwrite::<2>(1, [nb0.clone(), nb1.clone()]).await.unwrap();

  parser.restore(snapshot).await.unwrap();

  // verify replaced first block via mutate at current position 0:
  let expected_nb0 = nb0.clone();
  let got_nb0 = parser.mutate::<1, _>(async |blocks| Continue(blocks[0].clone())).await.unwrap().0;
  assert_eq!(expected_nb0, got_nb0);

  let expected_nb1 = nb1.clone();
  let got_nb1 = parser.mutate::<1, _>(async |blocks| Continue(blocks[0].clone())).await.unwrap().0;

  assert_eq!(expected_nb1, got_nb1);
}

/// Multiple-block overwrite replace only (no append); verify via reading.
#[tokio::test]
async fn overwrite_replace_two_blocks_then_read() {
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(1), lit(2), lit(3), lit(4), lit(5), lit(6), lit(7), lit(8)]).unwrap(),
  };
  let b1 = Block {
    operations: heapless::Vec::from_slice(&[lit(0xAA), srb(0x2A3, 6), lit(0xBB)]).unwrap(),
  };

  let mut parser = Yaz0Parser::new(prov(enc_blocks(&[b0.clone(), b1.clone()])), decoded_len_blocks(&[b0.clone(), b1.clone()]));

  let nb0 = Block {
    operations: heapless::Vec::from_slice(&[srb(0x010, 3), lit(0x99), lit(0x98), lit(0x97), lit(0x96), lit(0x95), lit(0x94), lit(0x93)]).unwrap(),
  };
  let nb1 = Block {
    operations: heapless::Vec::from_slice(&[lrb(0x020, 0x40), lit(0xCC), srb(0x005, 5)]).unwrap(),
  };

  let snapshot = parser.snapshot();

  parser.overwrite::<2>(2, [nb0.clone(), nb1.clone()]).await.unwrap();

  parser.restore(snapshot).await.unwrap();

  // Now reading should yield the replaced blocks in order.
  let r0 = parser.read(CLONED).await.unwrap();
  let r1 = parser.read(CLONED).await.unwrap();
  assert_eq!(r0.operations, nb0.operations);
  assert_eq!(r1.operations, nb1.operations);
}

/// Idempotent mutate: doing nothing must not change what the next read sees.
#[tokio::test]
async fn mutate_noop_preserves_next_block() {
  let b0 = Block {
    operations: heapless::Vec::from_slice(&[lit(9), srb(0x010, 3), lrb(0x020, 0x40), lit(8), lit(7), srb(0x111, 5), lit(6), lit(5)]).unwrap(),
  };
  let bytes = enc_blocks(&[b0.clone()]);
  let mut parser = Yaz0Parser::new(prov(bytes), decoded_len_blocks(&[b0.clone()]));

  let snapshot = parser.snapshot();

  parser.mutate::<1, _>(async move |_blocks| {}).await.unwrap();

  parser.restore(snapshot).await.unwrap();

  let r = parser.read(CLONED).await.unwrap();
  assert_eq!(r.operations, b0.operations);
}
