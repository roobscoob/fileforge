use proc_macro::TokenStream;

mod fileforge_error;
mod story;
mod text;

#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
  text::text(input)
}

#[proc_macro]
pub fn with_text(input: TokenStream) -> TokenStream {
  text::with_text(input)
}

#[proc_macro_attribute]
pub fn story(meta: TokenStream, item: TokenStream) -> TokenStream {
  story::story(meta, item)
}

#[proc_macro]
#[doc(hidden)]
pub fn dr(input: TokenStream) -> TokenStream {
  input
}

#[proc_macro]
#[doc(hidden)]
pub fn dv(input: TokenStream) -> TokenStream {
  input
}

#[proc_macro_derive(FileforgeError, attributes(from, report, flag, info))]
pub fn derive_fileforge_error(input: TokenStream) -> TokenStream {
  fileforge_error::derive(input)
}
