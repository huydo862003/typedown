//! Macros for the salsa database layer in typedown-db

use proc_macro::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn query_db_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

  let struct_name = &struct_ast.ident;

  // Require the struct to have a `storage` field
  let storage_field = struct_ast
    .fields
    .iter()
    .find(|field| field.ident.as_ref().is_some_and(|name| name == "storage"));

  // Get the type the user wrote for the `storage` field
  let storage_ty = match storage_field {
    Some(field) => &field.ty,
    None => {
      return syn::Error::new_spanned(&struct_ast, "expected a `storage: QueryStorage` field")
        .to_compile_error()
        .into();
    }
  };

  quote! {
    #struct_ast

    // Compile-time check: the `storage` field must be `typedown_db::QueryStorage`
    #[cfg(debug_assertions)]
    const _: () = <#storage_ty>::__TYPEDOWN_QUERY_STORAGE;

    impl typedown_db::QueryDatabase for #struct_name {}
  }
  .into()
}

pub fn query_input_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

  let visibility = &struct_ast.vis;
  let struct_name = &struct_ast.ident;

  quote! {
    // Validate the annotated struct is Send + Sync
    const _: () = {
      fn assert_send_sync<T: Send + Sync>() {}
      assert_send_sync::<#struct_name>();
    };

    #[cfg(debug_assertions)]
    const _: () = <#struct_name as typedown_db::InputId>::__TYPEDOWN_INPUT_ID; // validate that we actually refer to the correct struct

    #visibility struct #struct_name(usize);

    impl typedown_db::InputId for #struct_name {}
  }
  .into()
}

pub fn query_derived_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

  let visibility = &struct_ast.vis;
  let struct_name = &struct_ast.ident;

  quote! {
    // Validate the annotated struct is Send + Sync
    const _: () = {
      fn assert_send_sync<T: Send + Sync>() {}
      assert_send_sync::<#struct_name>();
    };

    #[cfg(debug_assertions)]
    const _: () = <#struct_name as typedown_db::DerivedId>::__TYPEDOWN_DERIVED_ID;

    #visibility struct #struct_name(usize);

    impl typedown_db::DerivedId for #struct_name {}
  }
  .into()
}
