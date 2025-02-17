use proc_macro::TokenStream;

mod text;
mod story;

#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
  text::text(input)
}

#[proc_macro_attribute]
pub fn story(meta: TokenStream, item: TokenStream) -> TokenStream {
  story::story(meta, item)
}

#[proc_macro]
#[doc(hidden)]
pub fn dr(input: TokenStream) -> TokenStream { input }
#[proc_macro]
#[doc(hidden)]
pub fn dv(input: TokenStream) -> TokenStream { input }
