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
  let struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

  let visibility = &struct_ast.vis;
  let struct_name = &struct_ast.ident;

  let fields: Vec<_> = match &struct_ast.fields {
    syn::Fields::Named(fields) => &fields.named,
    _ => {
      return syn::Error::new_spanned(&struct_ast, "expected a struct with named fields")
        .to_compile_error()
        .into();
    }
  }
  .iter()
  .collect();

  let mut output: TokenStream = quote! {}.into();

  for field in &fields {
    let field_ty = &field.ty;
    // Validate that every field is Send + Sync + Clone
    output.extend::<TokenStream>(
      quote! {
        const _: () = {
          const fn assert_send<T: Send>() {}
          const fn assert_sync<T: Sync>() {}
          const fn assert_clone<T: Clone>() {}
          assert_send::<#field_ty>();
          assert_sync::<#field_ty>();
          assert_clone::<#field_ty>();

          // Validate that InputFieldIngredient is what it is supposed to be
          #[cfg(debug_assertions)]
          const _: () = <typedown_db::InputFieldIngredient<#field_ty>>::__TYPEDOWN_INPUT_FIELD_INGREDIENT;

          // Validate that QueryStorage is what it is supposed to be
          #[cfg(debug_assertions)]
          const _: () = typedown_db::QueryStorage::__TYPEDOWN_QUERY_STORAGE;
        };
      }
      .into(),
    );
  }

  let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();
  let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
  let field_indices: Vec<_> = (0..fields.len()).collect();

  // Generate ingredients for all fields
  output.extend::<TokenStream>(
    quote! {
      typedown_db::inventory::submit! {
        typedown_db::Inventory {
          kind: typedown_db::IngredientKind::Input,
          register: |factories| {
            let start_index = factories.len();
            #(
              factories.push(|| Box::new(typedown_db::InputFieldIngredient::<#field_types>::new()));
            )*
            #struct_name::set_ingredient_start_index(start_index);
          },
        }
      }
    }
    .into(),
  );

  // Generate getters and setters
  let mut getter_setter_tokens = quote! {};
  for (idx, field) in fields.iter().enumerate() {
    let field_name = field.ident.as_ref().unwrap();
    let field_ty = &field.ty;
    let setter_name = quote::format_ident!("set_{}", field_name);

    getter_setter_tokens.extend(quote! {
      pub fn #field_name<DB: typedown_db::QueryDatabase>(&self, db: &DB) -> #field_ty {
        let storage = unsafe { db.storage() };
        let ingredient = storage.inputs[Self::ingredient_start_index() + #idx]
          .downcast_ref::<typedown_db::InputFieldIngredient<#field_ty>>().expect("ingredient type mismatch");
        let entry = ingredient.data.get(&self.0).expect("invalid input id");
        entry.value().clone()
      }

      pub fn #setter_name<DB: typedown_db::QueryDatabase>(&self, db: &mut DB, value: #field_ty) {
        let storage = unsafe { db.storage() };
        let ingredient = storage.inputs[Self::ingredient_start_index() + #idx]
          .downcast_ref::<typedown_db::InputFieldIngredient<#field_ty>>().expect("ingredient type mismatch");
        let mut entry = ingredient.data.get_mut(&self.0).expect("invalid input id");
        *entry.value_mut() = value;

        // We don't need to lock here
        // We expect that the Rust borrow checker would only allow one &mut db while no other &db is present
        // We just want a race-free revision counter here to signal "staleness" to later reads
        storage.revision.fetch_add(1, std::sync::atomic::Ordering::Release);
      }
    });
  }

  output.extend::<TokenStream>(
    quote! {
      #[derive(Clone, Copy, PartialEq, Eq, Hash)]
      #visibility struct #struct_name(usize);

      impl #struct_name {
        fn ingredient_start_index_lock() -> &'static std::sync::OnceLock<usize> {
          static START_INDEX: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
          &START_INDEX
        }

        fn ingredient_start_index() -> usize {
          *Self::ingredient_start_index_lock().get()
            .expect("ingredient not registered; was QueryStorage initialized?")
        }

        #[doc(hidden)]
        pub fn set_ingredient_start_index(index: usize) {
          let _ = Self::ingredient_start_index_lock().set(index);
        }

        fn next_id() -> usize {
          static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
          COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        }

        pub fn new<DB: typedown_db::QueryDatabase>(db: &DB, #(#field_names: #field_types),*) -> Self {
          let storage = unsafe { db.storage() };
          let id = Self::next_id();
          let start_index = Self::ingredient_start_index();

          #(
            {
              let ingredient = storage.inputs[start_index + #field_indices]
                .downcast_ref::<typedown_db::InputFieldIngredient<#field_types>>().expect("ingredient type mismatch");
              ingredient.data.insert(id, #field_names);
            }
          )*

          Self(id)
        }

        #getter_setter_tokens
      }

      impl typedown_db::InputId for #struct_name {}

      #[cfg(debug_assertions)]
      const _: () = <#struct_name as typedown_db::InputId>::__TYPEDOWN_INPUT_ID;
    }
    .into(),
  );

  output
}

pub fn query_derived_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
  // Only a struct can be decorated
  let struct_ast = match syn::parse::<ItemStruct>(item) {
    Ok(ast) => ast,
    Err(err) => return err.to_compile_error().into(),
  };

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

  let all_fields: Vec<_> = fields.iter().collect();

  // Generate a tuple type from all field types, e.g. (PathBuf, String)
  let field_types: Vec<_> = all_fields.iter().map(|field| &field.ty).collect();
  let data_tuple_ty = quote! { (#(#field_types,)*) };

  quote! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    #visibility struct #struct_name(usize);

    // Validate the data tuple is Send + Sync + Clone
    const _: () = {
      const fn assert_send<T: Send>() {}
      const fn assert_sync<T: Sync>() {}
      const fn assert_clone<T: Clone>() {}
      assert_send::<#data_tuple_ty>();
      assert_sync::<#data_tuple_ty>();
      assert_clone::<#data_tuple_ty>();
    };

    #[cfg(debug_assertions)]
    const _: () = <#struct_name as typedown_db::DerivedId>::__TYPEDOWN_DERIVED_ID;

    impl typedown_db::DerivedId for #struct_name {}
  }
  .into()
}
