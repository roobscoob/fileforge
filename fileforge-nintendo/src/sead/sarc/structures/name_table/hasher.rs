use core::hash::Hasher;

/// Hash mode for sead name table entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashMode {
  /// Modern sead: explicit i8 cast, consistent across platforms.
  /// This is also the behavior of legacy Clang builds (x86, AArch64).
  Signed,
  /// Legacy GHS/Wii U: unsigned char (u8).
  Unsigned,
}

/// A hasher that implements the sead name table hash algorithm.
///
/// The hash is computed as: `hash = hash * multiplier + byte`
/// where the signedness of `byte` depends on the `HashMode`.
#[derive(Debug, Clone, Copy)]
pub struct SfntHasher {
  hash: u32,
  multiplier: u32,
  mode: HashMode,
}

impl SfntHasher {
  /// Create a new hasher with the given multiplier and hash mode.
  pub fn new(multiplier: u32, mode: HashMode) -> Self {
    Self { hash: 0, multiplier, mode }
  }

  /// Create a new hasher with signed mode (modern sead).
  pub fn new_signed(multiplier: u32) -> Self {
    Self::new(multiplier, HashMode::Signed)
  }

  /// Create a new hasher with unsigned mode (legacy GHS/Wii U).
  pub fn new_unsigned(multiplier: u32) -> Self {
    Self::new(multiplier, HashMode::Unsigned)
  }

  /// Get the current hash value.
  pub fn get_hash(&self) -> u32 {
    self.hash
  }
}

impl Hasher for SfntHasher {
  fn finish(&self) -> u64 {
    self.hash as u64
  }

  fn write(&mut self, bytes: &[u8]) {
    match self.mode {
      HashMode::Signed => {
        for &byte in bytes {
          // Cast to i8 (signed char), which interprets bytes >= 128 as negative.
          // Then cast through i32 to properly sign-extend before converting to u32.
          // This matches C++ behavior: s8(str_[i]) -> int (implicit) -> u32 (implicit)
          //
          // Example: byte 0xFF
          //   as i8  -> -1 (0xFF interpreted as signed)
          //   as i32 -> -1 (0xFFFFFFFF in two's complement, sign extended)
          //   as u32 -> 4294967295 (0xFFFFFFFF, bit pattern preserved)
          let byte_signed = byte as i8; // Interpret as signed
          let extended = byte_signed as i32; // Sign-extend to 32-bit signed
          let as_u32 = extended as u32; // Convert to unsigned, preserving bit pattern

          self.hash = self.hash.wrapping_mul(self.multiplier).wrapping_add(as_u32);
        }
      }
      HashMode::Unsigned => {
        for &byte in bytes {
          // Use byte as-is (unsigned), zero-extended to u32
          // Example: byte 0xFF -> 255 as u32
          self.hash = self.hash.wrapping_mul(self.multiplier).wrapping_add(byte as u32);
        }
      }
    }
    // Mask to 32 bits (wrapping_mul/wrapping_add already handle this)
    self.hash &= 0xFFFFFFFF;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::hash::Hash;

  #[test]
  fn test_unsigned_mode() {
    let mut hasher = SfntHasher::new_unsigned(31);
    "test".hash(&mut hasher);
    let hash = hasher.finish();

    // Manual calculation: 0*31+116 = 116, 116*31+101 = 3697, etc.
    let expected = {
      let mut h = 0u32;
      for &b in b"test" {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
      }
      h & 0xFFFFFFFF
    };
    assert_eq!(hash as u32, expected);
  }

  #[test]
  fn test_signed_mode() {
    let mut hasher = SfntHasher::new_signed(31);
    "test".hash(&mut hasher);
    let hash = hasher.finish();

    // For ASCII this should match unsigned
    let mut hasher_unsigned = SfntHasher::new_unsigned(31);
    "test".hash(&mut hasher_unsigned);
    assert_eq!(hash, hasher_unsigned.finish());
  }

  #[test]
  fn test_non_ascii_difference() {
    // Test with a byte that has high bit set (>= 128)
    let data = &[0xFF]; // -1 as i8, 255 as u8

    let mut hasher_signed = SfntHasher::new_signed(31);
    hasher_signed.write(data);

    let mut hasher_unsigned = SfntHasher::new_unsigned(31);
    hasher_unsigned.write(data);

    // These should differ because 0xFF as i8 = -1, which sign-extends
    assert_ne!(hasher_signed.finish(), hasher_unsigned.finish());

    // Signed: 0xFF -> -1 as i8 -> 0xFFFFFFFF as u32
    assert_eq!(hasher_signed.finish() as u32, 0xFFFFFFFF);
    // Unsigned: 0xFF -> 255 as u32
    assert_eq!(hasher_unsigned.finish() as u32, 0xFF);
  }

  #[test]
  fn test_sign_extension_explicit() {
    // Explicitly test the sign extension behavior matches C++:
    // result = result * key + s8(str_[i])

    let test_cases = vec![
      // (byte, expected_signed_value_as_u32, expected_unsigned_value_as_u32)
      (0x00, 0x00000000, 0x00000000), // 0 in both
      (0x7F, 0x0000007F, 0x0000007F), // 127 in both (max positive i8)
      (0x80, 0xFFFFFF80, 0x00000080), // -128 vs 128
      (0xFF, 0xFFFFFFFF, 0x000000FF), // -1 vs 255
      (0xC0, 0xFFFFFFC0, 0x000000C0), // -64 vs 192
    ];

    for (byte, expected_signed, expected_unsigned) in test_cases {
      let mut hasher_signed = SfntHasher::new_signed(1); // multiplier=1 for simplicity
      hasher_signed.write(&[byte]);
      assert_eq!(
        hasher_signed.finish() as u32,
        expected_signed,
        "Signed mode: byte 0x{:02X} should produce 0x{:08X}",
        byte,
        expected_signed
      );

      let mut hasher_unsigned = SfntHasher::new_unsigned(1);
      hasher_unsigned.write(&[byte]);
      assert_eq!(
        hasher_unsigned.finish() as u32,
        expected_unsigned,
        "Unsigned mode: byte 0x{:02X} should produce 0x{:08X}",
        byte,
        expected_unsigned
      );
    }
  }

  #[test]
  fn test_matches_cpp_implementation() {
    // Simulate the C++ implementation:
    // u32 result = 0;
    // for (s32 i = 0; str_[i] != '\0'; i++)
    //     result = result * key + s8(str_[i]);

    fn cpp_style_hash(bytes: &[u8], key: u32) -> u32 {
      let mut result = 0u32;
      for &byte in bytes {
        let s8_val = byte as i8; // s8(str_[i])
        let as_i32 = s8_val as i32; // implicit conversion to int
        let as_u32 = as_i32 as u32; // implicit conversion to u32
        result = result.wrapping_mul(key).wrapping_add(as_u32);
      }
      result
    }

    let test_cases: Vec<(&[u8], u32)> = vec![
      (b"hello", 31),
      (b"test", 37),
      (&[0xFF, 0x80, 0x00, 0x7F], 31), // Mix of high and low bytes
      ("日本語".as_bytes(), 31),       // UTF-8 (will have bytes >= 128)
    ];

    for (input, key) in test_cases {
      let expected = cpp_style_hash(input, key);

      let mut hasher = SfntHasher::new_signed(key);
      hasher.write(input);

      assert_eq!(hasher.finish() as u32, expected, "Hash mismatch for input {:?} with key {}", input, key);
    }
  }

  #[test]
  fn test_wrapping() {
    let mut hasher = SfntHasher::new_unsigned(0xFFFFFFFF);
    hasher.write(&[0xFF]);
    // Should wrap without panicking
    let _ = hasher.finish();
  }
}
