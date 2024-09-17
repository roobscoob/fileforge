use fileforge_lib::{diagnostic::{node::name::DiagnosticNodeName, pool::{entry::DiagnosticPoolEntry, DiagnosticPool}}, object::{magic::Magic, nintendo::byml::unmanaged::UnmanagedByml}, provider::{builtin::rust_slice::RustSliceBinaryProvider, slice::fixed::FixedSliceProvider}, reader::{endianness::Endianness, Reader}, error::Error};

fn main() {
  let bytes = include_bytes!("../binaries/simple.byml");
  let provider = RustSliceBinaryProvider::over(bytes);
  let mut pool_buffer: [DiagnosticPoolEntry<32>; 32] = Default::default();
  let mut pool = DiagnosticPool::new(&mut pool_buffer);

  match UnmanagedByml::over(provider, &mut pool) {
    Err(e) => println!("Failed: {:?}", e.into_display()),
    Ok(mut byml) => match byml.version() {
      Err(e) => println!("Failed ver: {:?}", e.into_display()),
      Ok(v) => match byml.endianness() {
        Err(e) => println!("Failed end: {:?}", e.into_display()),
        Ok(end) => println!("Endianness: {end:?}\nVersion: {v}")
      },
    }
  }
}
