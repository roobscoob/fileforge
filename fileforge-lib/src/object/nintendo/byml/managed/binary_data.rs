use alloc::vec::Vec;

pub struct BinaryData {
  pub alignment: u32, // unknown the *fucking* wiki doesn't specify. Nintendo modders get your SHIT together
  pub data: Vec<u8>,
}