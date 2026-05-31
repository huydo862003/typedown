use proc_macro::TokenStream;
use quote::quote;
use syn::{
  Ident, Token,
  parse::{Parse, ParseStream},
  punctuated::Punctuated,
};

#[proc_macro_derive(AstNode)]
pub fn ast_node_derive(item: TokenStream) -> TokenStream {
  let item_ast: syn::DeriveInput = syn::parse(item).unwrap();

  let name = &item_ast.ident;
  let generated = quote! {
    impl AstNode for #name {
      fn cast(syntax: RedNode) -> Option<Self> {
          match syntax.kind() {
            typedown_types::syntax_kind::SyntaxKind::#name => Some(Self(syntax)),
            _ => None,
          }
        }
      fn syntax(&self) -> &RedNode {
        &self.0
      }
    };
  };
  generated.into()
}

// Parses: SyntaxKind = [Variant1, Variant2, ...]
struct WrapperArgs {
  kinds: Punctuated<Ident, Token![,]>,
}

impl Parse for WrapperArgs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let _kw: Ident = input.parse()?;
    let _eq: Token![=] = input.parse()?;
    let content;
    syn::bracketed!(content in input);
    let kinds = Punctuated::<Ident, Token![,]>::parse_terminated(&content)?;
    Ok(WrapperArgs { kinds })
  }
}

/// Attribute macro for synthetic AST nodes that wrap multiple SyntaxKind variants.
/// Usage:
/// ```
/// #[wrapper_ast_node(SyntaxKind = [KindA, KindB])]
/// pub struct MyNode(RedNode);
/// ```
#[proc_macro_attribute]
pub fn wrapper_ast_node(attr: TokenStream, item: TokenStream) -> TokenStream {
  let args: WrapperArgs = syn::parse(attr).unwrap();
  let item_ast: syn::DeriveInput = syn::parse(item).unwrap();

  let name = &item_ast.ident;
  let kinds: Vec<_> = args.kinds.iter().collect();

  let from_impls = kinds.iter().map(|kind| {
    quote! {
      impl From<#kind> for #name {
        fn from(node: #kind) -> Self {
          Self(node.syntax().clone())
        }
      }

      impl TryFrom<#name> for #kind {
        type Error = ();
        fn try_from(node: #name) -> Result<Self, ()> {
          Self::cast(node.0).ok_or(())
        }
      }
    }
  });

  let generated = quote! {
    #item_ast

    impl AstNode for #name {
      fn cast(syntax: RedNode) -> Option<Self> {
        match syntax.kind() {
          #(typedown_types::syntax_kind::SyntaxKind::#kinds)|* => Some(Self(syntax)),
          _ => None,
        }
      }
      fn syntax(&self) -> &RedNode {
        &self.0
      }
    }

    #(#from_impls)*
  };
  generated.into()
}
