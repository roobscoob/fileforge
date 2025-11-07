use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{collections::HashMap, str::FromStr};

use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
  parse::{Parse, ParseStream, Parser},
  punctuated::Punctuated,
  spanned::Spanned,
  ExprAssign, LitStr, Token,
};

struct Path(Option<TokenStream>, Vec<Segment>);

#[derive(Clone)]
struct Tag(TokenStream);

impl ToTokens for Tag {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.extend(self.0.clone());
  }
}

// Tag is now optional per segment.
struct Segment(Option<Tag>, LitStr);

fn parse_condition(input: ParseStream) -> syn::Result<TokenStream> {
  let group: Group = input.parse()?;

  if group.delimiter() != Delimiter::Brace {
    return Err(syn::Error::new(Span::call_site(), "expected {condition} group"));
  }

  Ok(group.stream())
}

fn parse_tag(input: ParseStream) -> syn::Result<Tag> {
  let group: Group = input.parse()?;

  if group.delimiter() != Delimiter::Bracket {
    return Err(syn::Error::new(Span::call_site(), "expected [tag] group"));
  }

  Ok(Tag(group.stream()))
}

/// Internal representation of what `text!` / `with_text!` need:
/// - remap_lets: the `let foo = expr;` bindings
/// - text_expr:  an expression that evaluates to a `Text`
struct TextExpansion {
  remap_lets: Vec<TokenStream>,
  text_expr: TokenStream,
}

/// Shared expansion logic for `text!` and `with_text!`.
fn expand_text_like(input: TokenStream) -> Result<TextExpansion, TokenStream> {
  let mut iter = input.into_iter().peekable();

  // Optional leading condition: `{cond}`
  let current_condition = if let Some(first) = iter.peek().cloned() {
    if let Some(condition) = parse_condition.parse2(first.to_token_stream()).ok() {
      // consume the condition token
      let _ = iter.next();
      Some(condition)
    } else {
      None
    }
  } else {
    None
  };

  // Tag is now optional and tracked as state.
  let mut current_tag: Option<Tag> = None;

  let mut conditions = vec![];
  let mut path = Path(current_condition, Vec::new());
  let mut logging = Vec::new();

  while let Some(token) = iter.next() {
    // Try to parse a [tag] group. If it works, update current_tag and continue.
    if let Ok(new_tag) = parse_tag.parse2(token.to_token_stream()) {
      current_tag = Some(new_tag);
      continue;
    }

    // Try to parse a string literal segment.
    if let Ok(str) = <LitStr as Parse>::parse.parse2(token.to_token_stream()) {
      path.1.push(Segment(current_tag.clone(), str));
      continue;
    }

    // Try to parse a comma, which might start a new conditional path.
    match <Token![,]>::parse.parse2(token.into_token_stream()) {
      Ok(_) => {
        let next = iter.peek().map(ToTokens::into_token_stream).unwrap_or_else(TokenStream::new);

        let res = Group::parse.parse2(next.clone()).is_ok();
        logging.push(format!("testing {res} {next:?}"));
        if path.1.len() > 0 {
          // you had an empty if-block here; keeping structure intact in case you
          // add logging / checks later.
        }

        if res {
          // New conditional branch: optional `{cond}` at start of next path.
          let new_condition = if let Some(condition) = parse_condition.parse2(next).ok() {
            // consume the condition group
            let _ = iter.next();
            Some(condition)
          } else {
            None
          };

          conditions.push(path);
          path = Path(new_condition, vec![]);
          // Note: we deliberately do NOT reset `current_tag` here; each path
          // has its own segments with tags set as needed via [tag] tokens.
          continue;
        } else {
          break;
        }
      }
      Err(error) => return Err(error.into_compile_error()),
    }
  }

  conditions.push(path);

  let mut remap_map: HashMap<String, TokenStream> = HashMap::new();

  let punct: Punctuated<ExprAssign, Token![,]> = match Punctuated::parse_terminated.parse2(iter.collect()) {
    Ok(punc) => punc,
    Err(error) => return Err(error.into_compile_error()),
  };

  for remap in punct {
    let str = match syn::Ident::parse.parse2(remap.left.to_token_stream()) {
      Ok(ident) => ident.to_string(),
      Err(error) => return Err(error.into_compile_error()),
    };

    if let Some(dup) = remap_map.insert(str, remap.right.into_token_stream()) {
      return Err(quote_spanned!(dup.span() => compile_error!("duplicate remapping")));
    }
  }

  let mut block = Vec::new();

  for Path(condition, segments) in conditions {
    let mut output = if std::env::var("CARGO_CRATE_NAME").is_ok_and(|v| v.eq("fileforge")) {
      quote!(crate::error::render::builtin::text::Text::new())
    } else {
      quote!(::fileforge::error::render::builtin::text::Text::new())
    };

    for Segment(tag, text) in segments {
      let text = text.value();
      let mut in_expr = 0;
      let mut start = 0;
      let iter = text.as_str().char_indices();

      for (index, ele) in iter {
        if ele == '{' {
          if in_expr == 0 {
            let substring = &text.as_str()[start..index];
            if !substring.is_empty() {
              if let Some(ref tag) = tag {
                output.extend(quote!(.push_tagged(#substring, #tag)));
              } else {
                // No tag: use untagged push
                output.extend(quote!(.push(#substring)));
              }
            }
            start = index + 1;
          }
          in_expr += 1;
        } else if ele == '}' {
          if in_expr == 0 {
            return Err(quote!(compile_error!("unbalanced amount of curly braces")));
          }

          in_expr -= 1;
          if in_expr == 0 {
            let substring = &text.as_str()[start..index];
            let tokens = remap_map.get(substring).cloned().unwrap_or_else(|| TokenStream::from_str(substring).unwrap());
            output.extend(quote!(.with(#tokens)));
            start = index + 1;
          }
        }
      }

      if in_expr != 0 {
        return Err(quote!(compile_error!("unbalanced amount of curly braces")));
      }

      let substring = &text.as_str()[start..];
      if !substring.is_empty() {
        if let Some(ref tag) = tag {
          output.extend(quote!(.push_tagged(#substring, #tag)));
        } else {
          output.extend(quote!(.push(#substring)));
        }
      }
    }

    let (pre, post) = if let Some(condition) = condition {
      (quote! { if #condition }, quote! { else })
    } else {
      (quote!(), quote!())
    };

    block.push(quote! {
      #pre {
        #output
      } #post
    });
  }

  // Turn the name→tokens map into actual `let` bindings
  let remap_lets: Vec<TokenStream> = remap_map
    .into_iter()
    .map(|(name, tokens)| (syn::Ident::new(&name, Span::call_site()), tokens))
    .map(|(name, tokens)| quote!( let #name = #tokens; ))
    .collect();

  // This is the expression that evaluates to a Text.
  // It is the same chain of blocks `text!` used to emit.
  let text_expr = quote! {
    // #(let _ = #logging;)*   // still here if you want debugging
    #(#block)*
  };

  Ok(TextExpansion { remap_lets, text_expr })
}

pub fn text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input_ts: TokenStream = input.into();

  match expand_text_like(input_ts) {
    Ok(TextExpansion { remap_lets, text_expr }) => quote! {
      {
        #(#remap_lets)*
        #text_expr
      }
    }
    .into(),
    Err(err) => err.into(),
  }
}

pub fn with_text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input_ts: TokenStream = input.into();

  let expansion = match expand_text_like(input_ts.clone()) {
    Ok(exp) => exp,
    Err(err) => return err.into(),
  };

  let TextExpansion { remap_lets, text_expr } = expansion;

  // Create a pseudo-random-ish, but deterministic, struct name based on the input.
  let mut hasher = DefaultHasher::new();
  input_ts.to_string().hash(&mut hasher);
  let hash = hasher.finish();
  let struct_ident = syn::Ident::new(&format!("Text{}", hash), Span::call_site());

  // Choose crate root: `crate` when compiling inside fileforge, `::fileforge` otherwise.
  let root = if std::env::var("CARGO_CRATE_NAME").is_ok_and(|v| v.eq("fileforge")) {
    quote!(crate)
  } else {
    quote!(::fileforge)
  };

  quote! {
    {
      struct #struct_ident;

      impl<'tag> #root::error::render::r#trait::renderable::WithRenderable<'tag> for #struct_ident {
        fn with<T>(callback: impl for<'a> FnOnce(&'a dyn #root::error::render::r#trait::renderable::Renderable<'tag>) -> T) -> T {
          #(#remap_lets)*

          let text = {
            #text_expr
          };

          callback(&text)
        }
      }

      #struct_ident
    }
  }
  .into()
}
