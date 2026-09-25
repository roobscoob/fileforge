//! `#[story("name", expr)]`: registers an example instance of an error type with the storybook.
//!
//! Inside `expr`, a made-up diagnostic tree can be described with two macros:
//! - `dr!(file / field)` becomes `Some(reference)` to the node `field` inside `file`.
//! - `dv!(file / field [value])` becomes a `DiagnosticValue` of `value`, located at `field`.
//!
//! Path segments are identifiers or string literals (`"save.bin" / "header.size"`). Any
//! segment may be placed with `@ start..end`, which sets its offset within its parent and
//! its size; a root only takes a size, so its range must start at 0. Each node may be
//! placed at most once. Unplaced nodes sit at offset 0 with an unknown size.
//!
//! Everything in `fileforge::storybook::prelude` is in scope inside `expr`.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
  bracketed,
  parse::{Parse, ParseStream, Parser},
  visit_mut::VisitMut,
  DeriveInput, Error, Expr, Ident, LitInt, LitStr, Token,
};

struct Placement {
  start: u64,
  end: u64,
  span: Span,
}

struct Segment {
  name: String,
  placement: Option<Placement>,
}

impl Parse for Segment {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name = if input.peek(LitStr) {
      input.parse::<LitStr>()?.value()
    } else {
      input.parse::<Ident>()?.to_string()
    };

    let placement = if input.peek(Token![@]) {
      let at: Token![@] = input.parse()?;
      let start: LitInt = input.parse()?;
      let _: Token![..] = input.parse()?;
      let end: LitInt = input.parse()?;
      let (start_value, end_value) = (start.base10_parse::<u64>()?, end.base10_parse::<u64>()?);

      if end_value < start_value {
        return Err(Error::new(end.span(), "the end of a placement must not be before its start"));
      }

      Some(Placement {
        start: start_value,
        end: end_value,
        span: at.span,
      })
    } else {
      None
    };

    Ok(Segment { name, placement })
  }
}

fn parse_path(input: ParseStream) -> syn::Result<Vec<Segment>> {
  let mut path = vec![input.parse()?];

  while input.peek(Token![/]) {
    let _: Token![/] = input.parse()?;
    path.push(input.parse()?);
  }

  Ok(path)
}

/// `dr!(path)`
struct Dr(Vec<Segment>);

impl Parse for Dr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    Ok(Dr(parse_path(input)?))
  }
}

/// `dv!(path [value])`
struct Dv {
  path: Vec<Segment>,
  value: Expr,
}

impl Parse for Dv {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let path = parse_path(input)?;
    let content;
    bracketed!(content in input);

    Ok(Dv { path, value: content.parse()? })
  }
}

struct Node {
  name: String,
  ident: Ident,
  parent: Option<usize>,
  placement: Option<(u64, u64)>,
}

/// The diagnostic tree described by every `dr!`/`dv!` in a story, in creation order.
struct Tree {
  nodes: Vec<Node>,
}

impl Tree {
  /// Returns the identifier bound to the node at the end of `path`, creating nodes as needed.
  fn insert(&mut self, path: Vec<Segment>) -> syn::Result<Ident> {
    let mut parent: Option<usize> = None;

    for segment in path {
      let existing = self.nodes.iter().position(|node| node.parent == parent && node.name == segment.name);

      let index = match existing {
        Some(index) => index,
        None => {
          self.nodes.push(Node {
            ident: format_ident!("__ff_story_node_{}", self.nodes.len()),
            name: segment.name.clone(),
            parent,
            placement: None,
          });
          self.nodes.len() - 1
        }
      };

      if let Some(placement) = segment.placement {
        if parent.is_none() && placement.start != 0 {
          return Err(Error::new(placement.span, "a root node has no offset, so its placement must start at 0"));
        }

        if self.nodes[index].placement.replace((placement.start, placement.end)).is_some() {
          return Err(Error::new(placement.span, format!("`{}` is placed more than once", segment.name)));
        }
      }

      parent = Some(index);
    }

    Ok(self.nodes[parent.expect("paths have at least one segment")].ident.clone())
  }

  fn to_tokens(&self, root: &TokenStream) -> TokenStream {
    let lets = self.nodes.iter().map(|node| {
      let Node { name, ident, parent, placement } = node;
      let (offset, size) = match placement {
        Some((start, end)) => (quote!(#start), quote!(::core::option::Option::Some(#end - #start))),
        None => (quote!(0), quote!(::core::option::Option::None)),
      };

      match parent {
        Some(parent) => {
          let parent = &self.nodes[*parent].ident;
          quote!(let #ident = #parent.create_physical_child(#offset, #size, #name);)
        }
        None => quote! {
          let #ident = #root::diagnostic::pool::DiagnosticPoolBuilder::create(&pool, #root::diagnostic::node::branch::DiagnosticBranch::None, #size, #name);
        },
      }
    });

    quote!(#(#lets)*)
  }
}

struct Visitor<'r> {
  root: &'r TokenStream,
  tree: Tree,
  errors: Vec<Error>,
}

impl<'r> VisitMut for Visitor<'r> {
  fn visit_expr_mut(&mut self, expr: &mut Expr) {
    let Expr::Macro(mac) = expr else {
      return syn::visit_mut::visit_expr_mut(self, expr);
    };

    let root = self.root;

    let replacement = if mac.mac.path.is_ident("dr") {
      Dr::parse
        .parse2(mac.mac.tokens.clone())
        .and_then(|Dr(path)| self.tree.insert(path))
        .map(|node| quote!(::core::option::Option::Some(#node)))
    } else if mac.mac.path.is_ident("dv") {
      Dv::parse.parse2(mac.mac.tokens.clone()).and_then(|Dv { path, value }| {
        let node = self.tree.insert(path)?;
        Ok(quote!(#root::diagnostic::value::DiagnosticValue(#value, ::core::option::Option::Some(#node))))
      })
    } else {
      return syn::visit_mut::visit_expr_mut(self, expr);
    };

    match replacement.and_then(syn::parse2::<Expr>) {
      Ok(replacement) => *expr = replacement,
      Err(error) => self.errors.push(error),
    }
  }
}

struct StoryMeta {
  name: LitStr,
  expr: Expr,
}

impl Parse for StoryMeta {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let name = input.parse()?;
    let _: Token![,] = input.parse()?;
    let expr = input.parse()?;
    let _: Option<Token![,]> = input.parse()?;

    Ok(StoryMeta { name, expr })
  }
}

fn fileforge_root() -> TokenStream {
  if std::env::var("CARGO_CRATE_NAME").is_ok_and(|name| name == "fileforge") {
    quote!(crate)
  } else {
    quote!(::fileforge)
  }
}

pub fn story(meta: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let item = TokenStream::from(item);
  let root = fileforge_root();

  let type_name = match DeriveInput::parse.parse2(item.clone()) {
    Ok(input) => input.ident.to_string(),
    Err(error) => return error.into_compile_error().into(),
  };

  let StoryMeta { name, mut expr } = match StoryMeta::parse.parse2(meta.into()) {
    Ok(meta) => meta,
    Err(error) => return error.into_compile_error().into(),
  };

  let mut visitor = Visitor {
    root: &root,
    tree: Tree { nodes: Vec::new() },
    errors: Vec::new(),
  };
  visitor.visit_expr_mut(&mut expr);

  if !visitor.errors.is_empty() {
    let errors = visitor.errors.iter().map(Error::to_compile_error);
    return quote!(#(#errors)* #item).into();
  }

  let nodes = visitor.tree.to_tokens(&root);

  quote! {
    #[cfg(feature = "story")]
    const _: () = {
      #root::storybook::inventory::submit! {
        #root::storybook::Story {
          name: #name,
          type_name: #type_name,
          module: ::core::module_path!(),
          render: |mode, width| {
            #[allow(unused_imports)]
            use #root::storybook::prelude::*;

            let pool = #root::diagnostic::pool::dynamic::DynamicDiagnosticPool::new();
            #nodes
            let error = #expr;
            #root::storybook::render(&error, &pool, mode, width)
          },
        }
      }
    };

    #item
  }
  .into()
}
