use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
  bracketed,
  parse::{Parse, ParseStream, Parser, Peek},
  parse_quote, DeriveInput, Expr, Ident, LitStr, Token,
};

fn parse_two_or_more<T: Parse, P>(input: ParseStream, peek: impl Fn(ParseStream) -> bool, separator: P) -> syn::Result<Vec<T>>
where
  P: Peek,
  P::Token: Parse,
{
  let mut list = vec![input.parse()?];
  let _: P::Token = input.parse()?;
  list.push(input.parse()?);
  while input.peek(separator) {
    let _: P::Token = input.parse()?;
    if input.is_empty() || peek(input) {
      break;
    }
    list.push(input.parse()?);
  }
  Ok(list)
}

struct DvPath(Vec<Ident>);
impl Parse for DvPath {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok(DvPath(parse_two_or_more(input, |input| input.peek(Token![/]), Token![/])?))
  }
}

struct Dv {
  path: DvPath,
  expr: Expr,
}
impl Parse for Dv {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let path = input.parse()?;
    let content;
    bracketed!(content in input);
    Ok(Dv { path, expr: content.parse()? })
  }
}

struct TreeEntry {
  ident: Ident,
  child_info: Option<(Ident, Expr)>,
  children: HashMap<Ident, TreeEntry>,
}

impl TreeEntry {
  pub fn new(ident: Ident) -> Self {
    Self {
      ident,
      child_info: None,
      children: HashMap::new(),
    }
  }

  pub fn insert_root(&mut self, path: &[Ident], expr: Expr) -> Result<(), ()> {
    self.children.entry(path[1].clone()).or_insert_with(|| TreeEntry::new(path[1].clone())).insert(&path, expr)
  }

  pub fn insert(&mut self, path: &[Ident], expr: Expr) -> Result<(), ()> {
    if path.len() == 2 {
      match self.child_info.replace((path[0].clone(), expr)) {
        Some(_) => Err(()),
        None => Ok(()),
      }
    } else {
      self.children.entry(path[0].clone()).or_insert_with(|| TreeEntry::new(path[0].clone())).insert(&path[1..], expr)
    }
  }
}

impl ToTokens for TreeEntry {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Self { ident, child_info, children } = self;
    let ident_string = ident.to_string();
    let expr = match child_info {
      Some((parent, _)) => quote! {
        #parent.create_physical_child(0, Some(0), crate::storybook::imports::DiagnosticNodeName::from(#ident_string))
      },
      None => {
        quote! {
          crate::storybook::imports::DiagnosticPool::create(&pool, crate::storybook::imports::DiagnosticBranch::None, Some(0), crate::storybook::imports::DiagnosticNodeName::from(#ident_string))
        }
      }
    };
    let children = children.values();
    tokens.extend(quote! {
      let #ident = #expr;
      #(#children)*
    });
  }
}

#[derive(Default)]
struct Visitor {
  diagnostics: HashMap<Ident, TreeEntry>,
  errors: Vec<TokenStream>,
}

impl syn::visit_mut::VisitMut for Visitor {
  fn visit_expr_macro_mut(&mut self, expr: &mut syn::ExprMacro) {
    if expr.mac.path.is_ident("dv") {
      match Dv::parse.parse2(expr.mac.tokens.clone()) {
        Ok(Dv { path: DvPath(path), expr: dv_expr }) => {
          let entry = self.diagnostics.entry(path[0].clone()).or_insert_with(|| TreeEntry::new(path[0].clone()));

          if let Err(()) = entry.insert_root(&path, dv_expr.clone()) {
            self.errors.push(quote! {
              {
                ::core::compile_error!("failed to insert!");
              }
            });
          } else {
            let last = path.last().unwrap();
            let path = &expr.mac.path;
            let bang = expr.mac.bang_token;
            *expr = parse_quote!(::fileforge_macros::#path #bang(crate::storybook::imports::DiagnosticValue(#dv_expr, Some(#last))))
          }
        }
        Err(error) => {
          let error = error.into_compile_error();
          self.errors.push(quote! {
            {
              #error
            }
          });
        }
      }
    } else if expr.mac.path.is_ident("dr") {
      match Ident::parse.parse2(expr.mac.tokens.clone()) {
        Ok(ident) => {
          let path = &expr.mac.path;
          let bang = expr.mac.bang_token;
          *expr = parse_quote!(::fileforge_macros::#path #bang(Some(#ident)));
          self.diagnostics.entry(ident.clone()).or_insert_with(|| TreeEntry::new(ident));
        }
        Err(error) => {
          let error = error.into_compile_error();
          self.errors.push(quote! {
            {
              #error
            }
          });
        }
      }
    } else {
      syn::visit_mut::visit_expr_macro_mut(self, expr);
    }
  }
}

struct MacroMeta {
  name: LitStr,
  expr: Expr,
}
impl Parse for MacroMeta {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name = input.parse()?;
    let _: Token![,] = input.parse()?;
    let expr = input.parse()?;

    Ok(MacroMeta { name, expr })
  }
}

pub fn story(meta: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item = TokenStream::from(item);
  let mut meta = TokenStream::from(meta);

  let type_name = DeriveInput::parse
    .parse2(item.clone())
    .map(|input| input.ident.to_string().to_token_stream())
    .unwrap_or_else(|error| error.into_compile_error());

  let mut story_name = quote!(::core::compile_error!("no story name provided"));

  let diagnostics = MacroMeta::parse
    .parse2(meta.clone())
    .ok()
    .map(|mut meta_input| {
      let mut visitor = Visitor::default();
      syn::visit_mut::visit_expr_mut(&mut visitor, &mut meta_input.expr);
      meta = meta_input.expr.into_token_stream();
      story_name = meta_input.name.into_token_stream();
      visitor.errors.into_iter().chain(visitor.diagnostics.into_values().map(ToTokens::into_token_stream))
    })
    .into_iter()
    .flatten();

  quote! {
    #[allow(non_snake_case)]
    #[cfg(feature = "story")]
    const _: () = {
      ::inventory::submit! {
        crate::storybook::Story {
          name: #story_name,
          type_name: #type_name,
          story: |
            pool: crate::storybook::imports::FixedDiagnosticPool<{ crate::storybook::NODE_NAME_SIZE }>,
            callback: fn(crate::storybook::imports::Report<{ crate::storybook::NODE_NAME_SIZE }>)
          | {
            #(#diagnostics)*
            crate::storybook::imports::FileforgeError::render_into_report(&(#meta), callback);
            drop(pool);
          }
        }
      }
    };
    #item
  }
  .into()
}
