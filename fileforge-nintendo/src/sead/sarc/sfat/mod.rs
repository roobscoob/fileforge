pub mod entry;
pub mod header;
pub mod name_table;
pub mod stream;

// Note:
// there's a weird logic hole.
// if there's only one file in the SfatTable
// then i cannot validate hash collisions
