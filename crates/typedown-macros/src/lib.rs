use proc_macro::TokenStream;
use quote::quote;

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
