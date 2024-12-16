use std::{collections::HashMap, fmt::Write, str::FromStr};

use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
  parse::{Parse, ParseStream, Parser},
  punctuated::Punctuated,
  spanned::Spanned,
  ExprAssign, LitStr, Token,
};

struct Path(Option<TokenStream>, Vec<Segment>);
struct Segment(Tag, LitStr);

#[derive(Clone)]
struct Tag(TokenStream);
impl ToTokens for Tag {
  fn to_tokens(&self, tokens: &mut TokenStream) { tokens.extend(self.0.clone()); }
}

fn parse_condition(input: ParseStream) -> syn::Result<TokenStream> {
  let group: Group = input.parse()?;

  if group.delimiter() != Delimiter::Brace {
    return Err(syn::Error::new(Span::call_site(), "what why "));
  }

  Ok(group.stream())
}
fn parse_tag(input: ParseStream) -> syn::Result<Tag> {
  let group: Group = input.parse()?;

  if group.delimiter() != Delimiter::Bracket {
    return Err(syn::Error::new(Span::call_site(), "huh "));
  }

  Ok(Tag(group.stream()))
}

#[proc_macro]
pub fn text(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input: proc_macro2::TokenStream = input.into();

  let mut iter = input.into_iter().peekable();
  let Some(mut tag) = iter.next() else {
    return syn::Error::new(Span::call_site(), "no tag specified")
      .into_compile_error()
      .into();
  };

  let current_condition =
    if let Some(condition) = parse_condition.parse2(tag.to_token_stream()).ok() {
      let Some(new_tag) = iter.next() else {
        return syn::Error::new(Span::call_site(), "no tag specified")
          .into_compile_error()
          .into();
      };

      tag = new_tag;
      Some(condition)
    } else {
      None
    };

  let mut tag = match parse_tag.parse2(tag.into_token_stream()) {
    Ok(tag) => tag,
    Err(error) => return error.into_compile_error().into(),
  };

  let mut conditions = vec![];
  let mut path = Path(current_condition, Vec::new());
  let mut logging = Vec::new();

  while let Some(token) = iter.next() {
    // if let Ok(condition) = parse_condition.parse2(token.to_token_stream()) {
    //   path.0 = Some(condition);
    //   continue;
    // }

    if let Ok(new_tag) = parse_tag.parse2(token.to_token_stream()) {
      tag = new_tag;
      continue;
    }

    if let Ok(str) = <LitStr as Parse>::parse.parse2(token.to_token_stream()) {
      path.1.push(Segment(tag.clone(), str));
      continue;
    }

    match <Token![,]>::parse.parse2(token.into_token_stream()) {
      Ok(_) => {
        let next = iter
          .peek()
          .map(ToTokens::into_token_stream)
          .unwrap_or_else(TokenStream::new);
        let res = Group::parse.parse2(next.clone()).is_ok();
        logging.push(format!("testing {res} {next:?}"));
        if path.1.len() > 0 {}

        if res {
          let current_condition = if let Some(condition) = parse_condition.parse2(next).ok() {
            let Some(_) = iter.next() else {
              return syn::Error::new(Span::call_site(), "no tag specified")
                .into_compile_error()
                .into();
            };

            Some(condition)
          } else {
            None
          };
          conditions.push(path);
          path = Path(current_condition, vec![]);
          continue;
        } else {
          break;
        }
      }
      Err(error) => return error.into_compile_error().into(),
    }
  }

  conditions.push(path);

  let mut remaps: HashMap<String, TokenStream> = HashMap::new();

  let punct: Punctuated<ExprAssign, Token![,]> =
    match Punctuated::parse_terminated.parse2(iter.collect()) {
      Ok(punc) => punc,
      Err(error) => return error.into_compile_error().into(),
    };

  for remap in punct {
    let str = match syn::Ident::parse.parse2(remap.left.to_token_stream()) {
      Ok(ident) => ident.to_string(),
      Err(error) => return error.into_compile_error().into(),
    };

    if let Some(dup) = remaps.insert(str, remap.right.into_token_stream()) {
      return quote_spanned!(dup.span() => compile_error!("duplicate remapping")).into();
    }
  }

  let mut block = Vec::new();

  for Path(condition, segments) in conditions {
    
    let mut output = if std::env::var("CARGO_CRATE_NAME").is_ok_and(|v| v.eq("fileforge_lib")) {
      quote!(crate::error::render::builtin::text::Text::new()) 
    } else {
      quote!(::fileforge_lib::error::render::builtin::text::Text::new())
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
            output.extend(quote!(.push(#substring, #tag)));
            start = index + 1;
          }
          in_expr += 1;
        } else if ele == '}' {
          if in_expr == 0 {
            return quote!(compile_error!("unbalanced amount of curly braces")).into();
          }

          in_expr -= 1;
          if in_expr == 0 {
            let substring = &text.as_str()[start..index];
            let tokens = remaps
              .get(substring)
              .cloned()
              .unwrap_or_else(|| TokenStream::from_str(substring).unwrap());
            output.extend(quote!(.with(#tokens)));
            start = index + 1;
          }
        }
      }

      if in_expr != 0 {
        return quote!(compile_error!("unbalanced amount of curly braces")).into();
      }

      let substring = &text.as_str()[start..];
      if substring.len() > 0 {
        output.extend(quote!(.push(#substring, #tag)));
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

  let remaps = remaps
    .into_iter()
    .map(|(name, tokens)| (syn::Ident::new(&name, Span::call_site()), tokens))
    .map(|(name, tokens)| quote!( let #name = #tokens; ));

  quote! {
    {
      // #(let _ = #logging;)*
      #(#remaps)*
      #(#block)*
    }
  }
  .into()
}
