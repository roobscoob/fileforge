use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote, ToTokens};
use syn::{
  parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, Attribute, Data, DeriveInput, Error, ExprAssign, Fields, Ident, LitStr, Meta, Path as SynPath, Token, Type,
};

struct FromAttr {
  present: bool,
  bound: Option<SynPath>,
}

struct DelegatingVariantInfo {
  ident: Ident,
  ty: Type,
  from_attr: FromAttr,
}

enum ReportVariantPatternKind {
  Unit,
  Tuple,
  Struct,
}

struct FlagExpansion {
  /// let __ff_flag_expr_N = <expr>;
  bindings: Vec<TokenStream2>,
  /// Arguments passed to `text!(...)`, e.g.
  ///   "msg {bound}", bound = &__ff_flag_expr_0
  text_args: TokenStream2,
}

// NEW: add error_fields to carry field-level #[error(...)]
struct ReportVariantInfo {
  ident: Ident,
  pattern_kind: ReportVariantPatternKind,
  report_expr: TokenStream2,
  named_fields: Vec<Ident>, // only used for Struct variants
  flags: Vec<FlagExpansion>,
  infos: Vec<FlagExpansion>,
  error_fields: Vec<(Ident, FlagExpansion)>,
}

fn fileforge_root() -> TokenStream2 {
  if std::env::var("CARGO_CRATE_NAME").is_ok_and(|name| name == "fileforge") {
    quote!(crate)
  } else {
    quote!(::fileforge)
  }
}

fn parse_from_attr(attrs: &[Attribute]) -> syn::Result<FromAttr> {
  let mut result = FromAttr { present: false, bound: None };

  for attr in attrs {
    if !attr.path().is_ident("from") {
      continue;
    }

    if result.present {
      return Err(Error::new(attr.span(), "duplicate #[from] attribute on the same field"));
    }

    result.present = true;

    match &attr.meta {
      Meta::Path(_) => {
        // #[from]
      }
      Meta::List(list) => {
        if list.tokens.is_empty() {
          // #[from()]
        } else {
          let path: SynPath = syn::parse2(list.tokens.clone())?;
          result.bound = Some(path);
        }
      }
      Meta::NameValue(_) => {
        return Err(Error::new(attr.span(), "unsupported #[from] syntax; use #[from] or #[from(TraitPath)]"));
      }
    }
  }

  Ok(result)
}

/// Parse #[report(expr)] on a struct or enum variant.
fn parse_report_attr(attrs: &[Attribute]) -> syn::Result<Option<TokenStream2>> {
  let mut result: Option<TokenStream2> = None;

  for attr in attrs {
    if !attr.path().is_ident("report") {
      continue;
    }

    if result.is_some() {
      return Err(Error::new(attr.span(), "duplicate #[report] attribute on the same item"));
    }

    match &attr.meta {
      Meta::List(list) => {
        if list.tokens.is_empty() {
          return Err(Error::new(attr.span(), "empty #[report] attribute; expected #[report(expr)]"));
        } else {
          result = Some(list.tokens.clone());
        }
      }
      Meta::Path(_) | Meta::NameValue(_) => {
        return Err(Error::new(attr.span(), "unsupported #[report] syntax; use #[report(expr)]"));
      }
    }
  }

  Ok(result)
}

/// Take the inside of a text-like attribute (#[flag]/#[info]/#[error]), e.g.
///   [Tag] "text {some_num.format().base(16)}", some_num = expr
/// and:
/// - turn each `name = expr` into a `let __ff_flag_expr_N = expr;` binding
/// - rewrite the mapping to `name = &__ff_flag_expr_N`
/// - keep the rest of the tokens intact so we can feed them to `text!(...)`
fn expand_flag_text_like(tokens: TokenStream2) -> syn::Result<FlagExpansion> {
  // Split tokens into:
  //   head: everything before the first top-level comma
  //   tail: everything after the first top-level comma
  let mut head = TokenStream2::new();
  let mut tail = TokenStream2::new();
  let mut in_tail = false;

  for tt in tokens.into_iter() {
    if !in_tail {
      if let TokenTree::Punct(ref p) = tt {
        if p.as_char() == ',' {
          in_tail = true;
          continue; // skip the comma itself
        }
      }
      head.extend(TokenStream2::from(tt));
    } else {
      tail.extend(TokenStream2::from(tt));
    }
  }

  // First: parse the tail as `name = expr, other = expr2, ...`
  use std::collections::HashMap;

  let mut bindings: Vec<TokenStream2> = Vec::new();
  let mut new_assigns: Vec<TokenStream2> = Vec::new();
  let mut tail_map: HashMap<String, usize> = HashMap::new();

  if !tail.is_empty() {
    let assigns: Punctuated<ExprAssign, Token![,]> = Punctuated::parse_terminated.parse2(tail)?;

    for assign in assigns.into_iter() {
      // Left side must be an ident (like in your text! macro remaps)
      let ident: Ident =
        syn::parse2(assign.left.clone().into_token_stream()).map_err(|e| Error::new(assign.span(), format!("expected identifier on left of = in #[flag]/#[info]/#[error], got {e}")))?;

      let name = ident.to_string();
      let expr_ts = assign.right.into_token_stream();
      let idx = bindings.len();
      let binding_ident = format_ident!("__ff_flag_expr_{}", idx);

      // let __ff_flag_expr_N = <expr>;
      bindings.push(quote! {
        let #binding_ident = #expr_ts;
      });

      // Rewrite mapping to `name = &__ff_flag_expr_N`
      new_assigns.push(quote! {
        #ident = &#binding_ident
      });

      tail_map.insert(name, idx);
    }
  }

  // Second: scan HEAD for string literals and turn `{expr}` into bindings.
  // If `{something}` is a single ident that exists in `tail_map`, we *don't*
  // create a new binding: it refers to the tail binding.
  let mut rewritten_head = TokenStream2::new();

  for tt in head.into_iter() {
    match &tt {
      TokenTree::Literal(_) => {
        let ts = TokenStream2::from(tt.clone());
        if let Ok(lit_str) = syn::parse2::<LitStr>(ts.clone()) {
          let s = lit_str.value();
          let chars: Vec<char> = s.chars().collect();
          let mut i = 0;
          let mut out = String::new();

          while i < chars.len() {
            let c = chars[i];

            if c == '{' {
              // Handle escaped "{{"
              if i + 1 < chars.len() && chars[i + 1] == '{' {
                out.push('{');
                i += 2;
                continue;
              }

              // Start of {expr}
              let start = i + 1;
              let mut j = start;
              while j < chars.len() && chars[j] != '}' {
                j += 1;
              }

              if j == chars.len() {
                return Err(Error::new(lit_str.span(), "unclosed `{` in #[flag]/#[info]/#[error] string"));
              }

              let expr_str: String = chars[start..j].iter().collect();

              // Try to treat it as a single ident first
              if let Ok(ident) = syn::parse_str::<Ident>(&expr_str) {
                let name = ident.to_string();
                if let Some(&_) = tail_map.get(&name) {
                  // This placeholder corresponds to a tail binding.
                  // Keep `{name}` in the literal; text! will see the
                  // `name = &__ff_flag_expr_idx` mapping we already added.
                  out.push('{');
                  out.push_str(&name);
                  out.push('}');
                  i = j + 1;
                  continue;
                }
              }

              // Otherwise, treat as a real expression: create a new binding
              let expr_ts: TokenStream2 = syn::parse_str(&expr_str).map_err(|e| Error::new(lit_str.span(), format!("invalid expression in {{...}} in #[flag]/#[info]/#[error]: {e}")))?;

              let idx = bindings.len();
              let binding_ident = format_ident!("__ff_flag_expr_{}", idx);

              bindings.push(quote! {
                let #binding_ident = #expr_ts;
              });

              // Rewrite to {&__ff_flag_expr_N}
              out.push('{');
              out.push_str("&__ff_flag_expr_");
              out.push_str(&idx.to_string());
              out.push('}');

              i = j + 1;
              continue;
            } else if c == '}' {
              // Handle escaped "}}"
              if i + 1 < chars.len() && chars[i + 1] == '}' {
                out.push('}');
                i += 2;
                continue;
              }

              return Err(Error::new(lit_str.span(), "unmatched `}` in #[flag]/#[info]/#[error] string"));
            } else {
              out.push(c);
              i += 1;
            }
          }

          let new_lit = LitStr::new(&out, lit_str.span());
          rewritten_head.extend(new_lit.to_token_stream());
        } else {
          // Not a string literal we care about; keep as-is
          rewritten_head.extend(TokenStream2::from(tt.clone()));
        }
      }
      _ => {
        // Non-literal tokens in head are preserved as-is
        rewritten_head.extend(TokenStream2::from(tt.clone()));
      }
    }
  }

  let head = rewritten_head;

  // Rebuild what we'll feed into `text!(...)`:
  //   head
  //   [ , name = &__ff_flag_expr_0, name2 = &__ff_flag_expr_1, ... ]
  let mut text_args = TokenStream2::new();
  text_args.extend(head);

  if !new_assigns.is_empty() {
    text_args.extend(quote! { , });
    let len = new_assigns.len();
    for (i, assign_ts) in new_assigns.into_iter().enumerate() {
      text_args.extend(assign_ts);
      if i + 1 < len {
        text_args.extend(quote! { , });
      }
    }
  }

  Ok(FlagExpansion { bindings, text_args })
}

/// Generic parser for `#[flag(...)]` / `#[info(...)]`-style attributes.
fn parse_text_line_attrs(attrs: &[Attribute], attr_name: &str) -> syn::Result<Vec<FlagExpansion>> {
  let mut result = Vec::new();

  for attr in attrs {
    if !attr.path().is_ident(attr_name) {
      continue;
    }

    match &attr.meta {
      Meta::List(list) => {
        if list.tokens.is_empty() {
          return Err(Error::new(attr.span(), format!("empty #[{}] attribute; expected #[{}(...)]", attr_name, attr_name)));
        }
        let expansion = expand_flag_text_like(list.tokens.clone())?;
        result.push(expansion);
      }
      Meta::Path(_) | Meta::NameValue(_) => {
        return Err(Error::new(
          attr.span(),
          format!("unsupported #[{}] syntax; use #[{}(\"... {{name}} ...\", name = expr)]", attr_name, attr_name),
        ));
      }
    }
  }

  Ok(result)
}

/// Parse all #[flag(...)] attributes on an item/variant.
fn parse_flag_attrs(attrs: &[Attribute]) -> syn::Result<Vec<FlagExpansion>> {
  parse_text_line_attrs(attrs, "flag")
}

/// Parse all #[info(...)] attributes on an item/variant.
fn parse_info_attrs(attrs: &[Attribute]) -> syn::Result<Vec<FlagExpansion>> {
  parse_text_line_attrs(attrs, "info")
}

/// Parse #[error(...)] on *fields* (we'll only use this on struct-like enum variants).
fn parse_error_attrs(attrs: &[Attribute]) -> syn::Result<Vec<FlagExpansion>> {
  parse_text_line_attrs(attrs, "error")
}

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as DeriveInput);

  let DeriveInput { attrs, vis: _, ident, generics, data } = input;

  let fileforge = fileforge_root();

  let expanded = match data {
    Data::Struct(data_struct) => {
      // Disallow #[error] on plain structs for now (only enum #[report] variants supported).
      for field in data_struct.fields.iter() {
        for attr in &field.attrs {
          if attr.path().is_ident("error") {
            return Error::new(attr.span(), "#[error(..)] is only supported on enum variants with #[report(..)]").to_compile_error().into();
          }
        }
      }

      // Struct-level FileforgeError using #[report] and optional #[flag]/#[info].
      let report_expr = match parse_report_attr(&attrs) {
        Ok(Some(expr)) => expr,
        Ok(None) => {
          return Error::new_spanned(&ident, "FileforgeError for structs requires a #[report(expr)] attribute").to_compile_error().into();
        }
        Err(e) => return e.to_compile_error().into(),
      };

      let flags = match parse_flag_attrs(&attrs) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
      };

      let infos = match parse_info_attrs(&attrs) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
      };

      let generics_for_error = generics.clone();
      let (impl_generics, ty_generics, where_clause) = generics_for_error.split_for_impl();

      let mut flag_stmts = TokenStream2::new();
      let mut flag_chains = TokenStream2::new();

      for (i, flag) in flags.iter().enumerate() {
        let flag_ident = format_ident!("__ff_flag_line_{}", i);
        let bindings = &flag.bindings;
        let text_args = &flag.text_args;

        flag_stmts.extend(quote! {
          #(#bindings)*
          let #flag_ident = ::fileforge_macros::text!( #text_args );
        });

        flag_chains.extend(quote! {
          .with_flag_line(&#flag_ident)
        });
      }

      let mut info_stmts = TokenStream2::new();
      let mut info_chains = TokenStream2::new();

      for (i, info) in infos.iter().enumerate() {
        let info_ident = format_ident!("__ff_info_line_{}", i);
        let bindings = &info.bindings;
        let text_args = &info.text_args;

        info_stmts.extend(quote! {
          #(#bindings)*
          let #info_ident = ::fileforge_macros::text!( #text_args );
        });

        info_chains.extend(quote! {
          .with_info_line(&#info_ident)
        });
      }

      quote! {
        impl #impl_generics #fileforge::error::FileforgeError
            for #ident #ty_generics
            #where_clause
        {
          fn render_into_report<
            P: #fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone,
            const ITEM_NAME_SIZE: usize
          >(
            &self,
            provider: P,
            callback: impl for<'tag, 'b>
              FnOnce(#fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()
          ) {
            #flag_stmts
            #info_stmts

            #fileforge::error::report::Report::new::<Self>(provider, #report_expr)
              #flag_chains
              #info_chains
              .apply(callback)
          }
        }
      }
    }

    Data::Enum(data_enum) => {
      let mut delegating_variants = Vec::<DelegatingVariantInfo>::new();
      let mut report_variants = Vec::<ReportVariantInfo>::new();

      for variant in data_enum.variants {
        let v_ident = variant.ident.clone();

        let flags = match parse_flag_attrs(&variant.attrs) {
          Ok(f) => f,
          Err(e) => return e.to_compile_error().into(),
        };

        let infos = match parse_info_attrs(&variant.attrs) {
          Ok(f) => f,
          Err(e) => return e.to_compile_error().into(),
        };

        let report_attr = match parse_report_attr(&variant.attrs) {
          Ok(a) => a,
          Err(e) => return e.to_compile_error().into(),
        };

        let fields = variant.fields;

        if let Some(report_expr) = report_attr {
          // "report" variant: we own the report generation.
          match fields {
            Fields::Named(named) => {
              let field_idents: Vec<Ident> = named.named.iter().filter_map(|f| f.ident.clone()).collect();

              // NEW: collect field-level #[error(...)] attributes.
              let mut error_fields: Vec<(Ident, FlagExpansion)> = Vec::new();

              for field in named.named.iter() {
                let field_ident = field.ident.clone().unwrap();
                let errs = match parse_error_attrs(&field.attrs) {
                  Ok(e) => e,
                  Err(e) => return e.to_compile_error().into(),
                };

                if errs.len() > 1 {
                  return Error::new_spanned(field, "multiple #[error(..)] attributes on the same field are not supported")
                    .to_compile_error()
                    .into();
                }

                if let Some(err_expansion) = errs.into_iter().next() {
                  error_fields.push((field_ident.clone(), err_expansion));
                }
              }

              report_variants.push(ReportVariantInfo {
                ident: v_ident,
                pattern_kind: ReportVariantPatternKind::Struct,
                report_expr,
                named_fields: field_idents,
                flags,
                infos,
                error_fields,
              });
            }
            Fields::Unnamed(_) => {
              if !flags.is_empty() || !infos.is_empty() {
                return Error::new_spanned(v_ident, "#[flag]/#[info] on tuple-like #[report] variants is not supported yet")
                  .to_compile_error()
                  .into();
              }

              // For now, #[error] on tuple-like variants is not supported either.
              report_variants.push(ReportVariantInfo {
                ident: v_ident,
                pattern_kind: ReportVariantPatternKind::Tuple,
                report_expr,
                named_fields: Vec::new(),
                flags,
                infos,
                error_fields: Vec::new(),
              });
            }
            Fields::Unit => {
              if !flags.is_empty() || !infos.is_empty() {
                return Error::new_spanned(v_ident, "#[flag]/#[info] on unit #[report] variants is not supported yet").to_compile_error().into();
              }

              report_variants.push(ReportVariantInfo {
                ident: v_ident,
                pattern_kind: ReportVariantPatternKind::Unit,
                report_expr,
                named_fields: Vec::new(),
                flags,
                infos,
                error_fields: Vec::new(),
              });
            }
          }

          continue;
        }

        // No #[report] on this variant: it must be a delegating tuple-like variant.
        if !flags.is_empty() || !infos.is_empty() {
          return Error::new_spanned(v_ident, "#[flag]/#[info] is only supported on variants with #[report(..)] for now")
            .to_compile_error()
            .into();
        }

        // Disallow #[error] on delegating variants for now.
        if let Fields::Named(named) = &fields {
          for field in &named.named {
            for attr in &field.attrs {
              if attr.path().is_ident("error") {
                return Error::new(attr.span(), "#[error(..)] is only supported on enum variants with #[report(..)]").to_compile_error().into();
              }
            }
          }
        }

        match fields {
          Fields::Unnamed(unnamed) => {
            if unnamed.unnamed.len() != 1 {
              return Error::new_spanned(
                unnamed,
                "FileforgeError enum variants must be tuple-like with exactly one field \
                 (or use #[report(expr)] on the variant)",
              )
              .to_compile_error()
              .into();
            }

            let field = &unnamed.unnamed[0];
            let ty = field.ty.clone();

            let from_attr = match parse_from_attr(&field.attrs) {
              Ok(a) => a,
              Err(e) => return e.to_compile_error().into(),
            };

            delegating_variants.push(DelegatingVariantInfo { ident: v_ident, ty, from_attr });
          }
          _ => {
            return Error::new_spanned(
              v_ident,
              "FileforgeError enum variants must be tuple-like (e.g. Variant(T)) \
               or have #[report(expr)] on the variant",
            )
            .to_compile_error()
            .into();
          }
        }
      }

      // Add T: FileforgeError bounds for delegating variants.
      let mut generics = generics.clone();
      {
        let where_clause = generics.make_where_clause();
        for info in &delegating_variants {
          let ty = &info.ty;
          where_clause.predicates.push(parse_quote!(#ty: #fileforge::error::FileforgeError));
        }
      }

      let generics_for_error = generics.clone();
      let (impl_generics, ty_generics, where_clause) = generics_for_error.split_for_impl();

      // Delegating variants just forward to inner error.
      let delegating_match_arms = delegating_variants.iter().map(|info| {
        let v_ident = &info.ident;
        quote! {
          Self::#v_ident(inner) => inner.render_into_report(provider, callback),
        }
      });

      // #[report] variants generate new reports, plus flags/infos/errors if any.
      let report_match_arms = report_variants.iter().map(|info| {
        let v_ident = &info.ident;
        let report_expr = &info.report_expr;

        let mut flag_stmts = TokenStream2::new();
        let mut flag_chains = TokenStream2::new();

        for (i, flag) in info.flags.iter().enumerate() {
          let flag_ident = format_ident!("__ff_flag_line_{}", i);
          let bindings = &flag.bindings;
          let text_args = &flag.text_args;

          flag_stmts.extend(quote! {
            #(#bindings)*
            let #flag_ident = ::fileforge_macros::text!( #text_args );
          });

          flag_chains.extend(quote! {
            .with_flag_line(&#flag_ident)
          });
        }

        let mut info_stmts = TokenStream2::new();
        let mut info_chains = TokenStream2::new();

        for (i, info_attr) in info.infos.iter().enumerate() {
          let info_ident = format_ident!("__ff_info_line_{}", i);
          let bindings = &info_attr.bindings;
          let text_args = &info_attr.text_args;

          info_stmts.extend(quote! {
            #(#bindings)*
            let #info_ident = ::fileforge_macros::text!( #text_args );
          });

          info_chains.extend(quote! {
            .with_info_line(&#info_ident)
          });
        }

        // NEW: field-level #[error] handling for struct variants.
        let mut error_stmts = TokenStream2::new();
        let mut error_chains = TokenStream2::new();

        if !info.error_fields.is_empty() {
          for (i, (field_ident, expansion)) in info.error_fields.iter().enumerate() {
            let err_ident = format_ident!("__ff_error_text_{}", i);
            let bindings = &expansion.bindings;
            let text_args = &expansion.text_args;
            let field_name_str = field_ident.to_string();

            // Build the text!(...) for this field.
            error_stmts.extend(quote! {
              #(#bindings)*
              let #err_ident = ::fileforge_macros::text!( #text_args );
            });

            // Add context + contextual error note for this field.
            error_chains.extend(quote! {
              .with_context(#field_name_str, #field_ident)
              .with_contextual_note_or_info(
                #field_name_str,
                &#err_ident,
                |n| n.with_tag(&#fileforge::error::render::buffer::cell::tag::builtin::report::REPORT_ERROR_TEXT)
              )
            });
          }
        }

        match info.pattern_kind {
          ReportVariantPatternKind::Unit => {
            quote! {
              Self::#v_ident => {
                #flag_stmts
                #info_stmts
                #fileforge::error::report::Report::new::<Self>(provider, #report_expr)
                  #flag_chains
                  #info_chains
                  .apply(callback)
              }
            }
          }
          ReportVariantPatternKind::Tuple => {
            quote! {
              Self::#v_ident(..) => {
                #flag_stmts
                #info_stmts
                #fileforge::error::report::Report::new::<Self>(provider, #report_expr)
                  #flag_chains
                  #info_chains
                  .apply(callback)
              }
            }
          }
          ReportVariantPatternKind::Struct => {
            let field_idents = &info.named_fields;

            if info.error_fields.is_empty() {
              quote! {
                Self::#v_ident { #(#field_idents),* } => {
                  #flag_stmts
                  #info_stmts
                  #fileforge::error::report::Report::new::<Self>(provider, #report_expr)
                    #flag_chains
                    #info_chains
                    .apply(callback)
                }
              }
            } else {
              quote! {
                Self::#v_ident { #(#field_idents),* } => {
                  #flag_stmts
                  #info_stmts
                  #error_stmts
                  #fileforge::error::report::Report::new::<Self>(provider, #report_expr)
                    #flag_chains
                    .with_error_context()
                    #error_chains
                    .finalize_context()
                    #info_chains
                    .apply(callback)
                }
              }
            }
          }
        }
      });

      let match_arms = delegating_match_arms.chain(report_match_arms);

      // From<T> impls for #[from] variants.
      let from_impls = delegating_variants.iter().filter(|info| info.from_attr.present).map(|info| {
        let v_ident = &info.ident;
        let ty = &info.ty;

        let mut g = generics.clone();
        if let Some(bound_trait) = &info.from_attr.bound {
          let where_clause = g.make_where_clause();
          where_clause.predicates.push(parse_quote!(#ty: #bound_trait));
        }
        let (impl_g2, ty_g2, where2) = g.split_for_impl();

        quote! {
          impl #impl_g2 From<#ty> for #ident #ty_g2 #where2 {
            fn from(value: #ty) -> Self {
              Self::#v_ident(value)
            }
          }
        }
      });

      quote! {
        impl #impl_generics #fileforge::error::FileforgeError
            for #ident #ty_generics
            #where_clause
        {
          fn render_into_report<
            P: #fileforge::diagnostic::pool::DiagnosticPoolProvider + Clone,
            const ITEM_NAME_SIZE: usize
          >(
            &self,
            provider: P,
            callback: impl for<'tag, 'b>
              FnOnce(#fileforge::error::report::Report<'tag, 'b, ITEM_NAME_SIZE, P>) -> ()
          ) {
            match self {
              #(#match_arms)*
            }
          }
        }

        #(#from_impls)*
      }
    }

    Data::Union(u) => Error::new_spanned(u.union_token, "FileforgeError cannot be derived for unions").to_compile_error(),
  };

  expanded.into()
}
