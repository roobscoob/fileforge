use proc_macro::TokenStream;

mod story;
mod text;

#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
  text::text(input)
}

#[proc_macro_attribute]
pub fn story(meta: TokenStream, item: TokenStream) -> TokenStream {
  story::story(meta, item)
}
