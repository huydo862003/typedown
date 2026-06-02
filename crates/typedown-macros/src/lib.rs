use proc_macro::TokenStream;

mod ast;
mod db;

#[proc_macro_derive(AstNode)]
pub fn ast_node_derive(item: TokenStream) -> TokenStream {
  ast::ast_node_derive_impl(item)
}

/// Attribute macro for synthetic AST nodes that wrap multiple SyntaxKind variants.
/// Usage:
/// ```ignore
/// #[wrapper_ast_node(SyntaxKind = [KindA, KindB])]
/// pub struct MyNode(RedNode);
/// ```
#[proc_macro_attribute]
pub fn wrapper_ast_node(attr: TokenStream, item: TokenStream) -> TokenStream {
  ast::wrapper_ast_node_impl(attr, item)
}
