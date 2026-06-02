//! Macros for the salsa database layer in typeodown-db

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, Type};

pub fn query_db_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let struct_ast: ItemStruct = syn::parse(item).expect("query_db can only be applied to structs");
  let struct_name = &struct_ast.ident;

  // Require the struct to have a `storage` field
  let has_storage = struct_ast
    .fields
    .iter()
    .any(|field| field.ident.as_ref().is_some_and(|name| name == "storage"));

  if !has_storage {
    panic!("Expect the database struct to define a field named `storage`")
  }

  // Get the type the user wrote for the `storage` field
  let storage_ty = struct_ast
    .fields
    .iter()
    .find(|field| field.ident.as_ref().is_some_and(|name| name == "storage"))
    .map(|field| &field.ty)
    .unwrap();

  quote! {
    #struct_ast

    // Compile-time check: the `storage` field must be `typedown_db::QueryStorage`
    #[cfg(debug_assertions)]
    const _: () = <#storage_ty>::__TYPEDOWN_QUERY_STORAGE;

    impl typedown_db::QueryDatabase for #struct_name {}
  }
  .into()
}
