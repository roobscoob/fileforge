use crate::diagnostic::value::DiagnosticValue;

#[derive(Clone, Copy)]
pub struct OutOfBoundsError {
  pub read_offset: u64,
  pub read_length: Option<u64>,
  pub provider_size: u64,
}

impl OutOfBoundsError {
  pub fn assert(provider_size: u64, read_offset: u64, read_length: Option<u64>) -> Result<(), Self> {
    if let Some(read_length) = read_length {
      let read_end = read_offset.checked_add(read_length).ok_or(OutOfBoundsError {
        read_offset,
        read_length: Some(read_length),
        provider_size,
      })?;

      if read_end > provider_size {
        Err(Self {
          read_offset,
          read_length: Some(read_length),
          provider_size,
        })
      } else {
        Ok(())
      }
    } else {
      if read_offset > provider_size {
        Err(Self {
          read_offset,
          read_length: None,
          provider_size,
        })
      } else {
        Ok(())
      }
    }
  }
}
