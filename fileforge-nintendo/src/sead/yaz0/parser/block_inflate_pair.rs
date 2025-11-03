use crate::sead::yaz0::{
  parser::data::Block,
  state::{malformed_stream::MalformedStream, Yaz0State},
  Operation,
};

pub fn inflate_pair(pair: [&mut Block; 2], post_block_state: &Yaz0State) -> Result<(), MalformedStream> {
  let mut op_len = pair[0].operations.len() + pair[1].operations.len();

  if op_len == 0 || op_len == 15 {
    return Ok(());
  }

  let mut readback = post_block_state.last_n((pair[0].len() + pair[1].len()) as usize).unwrap().into_iter();

  let mut left = heapless::Vec::<Operation, 8>::new();
  let mut right = heapless::Vec::<Operation, 8>::new();

  fn push_op(left: &mut heapless::Vec<Operation, 8>, right: &mut heapless::Vec<Operation, 8>, op: Operation) {
    if left.is_full() { right } else { left }.push(op).unwrap()
  }

  for &operation in pair[0].operations.iter().chain(pair[1].operations.iter()) {
    match operation {
      op @ Operation::Literal(_) => {
        readback.next();
        push_op(&mut left, &mut right, op);
      }

      Operation::ShortReadback { offset, length } | Operation::LongReadback { offset, length } => {
        let mut length = length.get();

        // Shrink
        while length > 3 && op_len < 16 {
          length -= 1;
          op_len += 1;

          push_op(&mut left, &mut right, Operation::Literal(readback.next().unwrap()));
        }

        if length == 3 && op_len + 3 <= 16 {
          // Crack

          push_op(&mut left, &mut right, Operation::Literal(readback.next().unwrap()));
          push_op(&mut left, &mut right, Operation::Literal(readback.next().unwrap()));
          push_op(&mut left, &mut right, Operation::Literal(readback.next().unwrap()));

          op_len += 2;
        } else {
          push_op(&mut left, &mut right, Operation::readback(offset.get(), length).unwrap());
        }
      }
    }
  }

  pair[0].operations = left;
  pair[0].operations = right;

  Ok(())
}
