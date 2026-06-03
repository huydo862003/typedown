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

    impl typedown_db::QueryDatabase for #struct_name {
      unsafe fn storage(&self) -> &typedown_db::QueryStorage {
        &self.storage
      }

      unsafe fn storage_mut(&mut self) -> &mut typedown_db::QueryStorage {
        &mut self.storage
      }
    }
  }
  .into()
}

pub fn query_input_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let mut struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

  // Auto-derive Clone on the original struct so it can be stored in the database
  struct_ast.attrs.push(syn::parse_quote!(#[derive(Clone)]));

  let visibility = &struct_ast.vis;
  let struct_name = &struct_ast.ident;

  let fields = match &struct_ast.fields {
    syn::Fields::Named(fields) => &fields.named,
    _ => {
      return syn::Error::new_spanned(&struct_ast, "expected a struct with named fields")
        .to_compile_error()
        .into();
    }
  };

  // TIL: Fields decorated with macros will have them stored in .attrs
  let tracked_fields: Vec<_> = fields
    .iter()
    .filter(|field| {
      field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("tracked"))
    })
    .collect();

  let untracked_fields: Vec<_> = fields
    .iter()
    .filter(|field| {
      !field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("tracked"))
    })
    .collect();

  let all_fields: Vec<_> = fields.iter().collect();

  // Generate a tuple type from all field types, e.g. (PathBuf, String)
  let field_types: Vec<_> = all_fields.iter().map(|field| &field.ty).collect();
  let data_tuple_ty = quote! { (#(#field_types),*) };

  // Generate new() constructor
  let new_params = all_fields.iter().map(|field| {
    let field_name = field.ident.as_ref().unwrap();
    let field_ty = &field.ty;
    quote! { #field_name: #field_ty }
  });

  // Generate all getters for the struct
  let getters = all_fields.iter().map(|field| {
    let field_name = field.ident.as_ref().unwrap();
    let field_ty = &field.ty;
    quote! {
      pub fn #field_name<DB: typedown_db::QueryDatabase + 'db>(&self, db: &DB) -> #field_ty {
        todo!()
      }
    }
  });

  // Generate all setters for the struct
  let setters = all_fields.iter().map(|field| {
    let field_name = field.ident.as_ref().unwrap();
    let field_ty = &field.ty;
    let setter_name = quote::format_ident!("set_{}", field_name);
    quote! {
      pub fn #setter_name<DB: typedown_db::QueryDatabase + 'db>(&self, db: &mut DB, value: #field_ty) {
        todo!()
      }
    }
  });

  quote! {
    #visibility struct #struct_name<'db>(usize, std::marker::PhantomData<&'db ()>);

    // Validate the generated struct is Send + Sync + Clone
    const _: () = {
      #struct_ast

      const fn assert_send<T: Send>() {}
      const fn assert_sync<T: Sync>() {}
      const fn assert_clone<T: Clone>() {}
      assert_send::<#struct_name>();
      assert_sync::<#struct_name>();
      assert_clone::<#struct_name>();

      #[cfg(debug_assertions)]
      const _: () = <typedown_db::InputIngredient<#struct_name>>::__TYPEDOWN_INPUT_INGREDIENT;
    };

    #[cfg(debug_assertions)]
    const _: () = typedown_db::QueryStorage::__TYPEDOWN_QUERY_STORAGE;

    impl<'db> #struct_name<'db> {
      fn get_db_index () -> usize {
        static INDEX: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *INDEX.get_or_init(|| {
          typedown_db::QueryStorage::INPUT_INDEX.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        })
      }

      pub fn new<DB: typedown_db::QueryDatabase>(db: &'db DB, #(#new_params),*) -> Self {
        todo!()
      }

      #(#getters)*
      #(#setters)*
    }

    impl<'db> typedown_db::InputId<'db> for #struct_name<'db> {}

    #[cfg(debug_assertions)]
    const _: () = <#struct_name as typedown_db::InputId>::__TYPEDOWN_INPUT_ID; // validate that we actually refer to the correct struct
  }
  .into()
}

pub fn query_derived_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let mut struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

  // Auto-derive Clone on the original struct so it can be stored in the database
  struct_ast.attrs.push(syn::parse_quote!(#[derive(Clone)]));

  let visibility = &struct_ast.vis;
  let struct_name = &struct_ast.ident;

  quote! {
    #visibility struct #struct_name<'db>(usize, std::marker::PhantomData<&'db ()>);

    // Validate the generated struct is Send + Sync + Clone
    const _: () = {
      #struct_ast

      const fn assert_send<T: Send>() {}
      const fn assert_sync<T: Sync>() {}
      const fn assert_clone<T: Clone>() {}
      assert_send::<#struct_name>();
      assert_sync::<#struct_name>();
      assert_clone::<#struct_name>();
    };

    #[cfg(debug_assertions)]
    const _: () = <#struct_name as typedown_db::DerivedId>::__TYPEDOWN_DERIVED_ID;

    impl<'db> typedown_db::DerivedId<'db> for #struct_name<'db> {}
  }
  .into()
}
