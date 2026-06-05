//! Macros for the salsa database layer in typedown-db

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, ItemStruct};

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
        let ingredient_index = Self::ingredient_start_index() + #idx;
        let ingredient = (&*storage.ingredients[ingredient_index] as &dyn std::any::Any)
          .downcast_ref::<typedown_db::InputFieldIngredient<#field_ty>>().expect("ingredient type mismatch");
        let entry = ingredient.data.get(&self.0).expect("invalid input id");

        // Record dependency if inside a derived query
        storage.with_context(|ctx| {
          if let Some(ctx) = ctx {
            ctx.dependencies.push(typedown_db::Dependency {
              ingredient_index,
              arg_id: self.0,
              changed_at: entry.changed_at,
            });
          }
        });

        entry.value.clone()
      }

      pub fn #setter_name<DB: typedown_db::QueryDatabase>(&self, db: &mut DB, value: #field_ty) {
        let storage = unsafe { db.storage() };
        let ingredient = (&*storage.ingredients[Self::ingredient_start_index() + #idx] as &dyn std::any::Any)
          .downcast_ref::<typedown_db::InputFieldIngredient<#field_ty>>().expect("ingredient type mismatch");
        let mut entry = ingredient.data.get_mut(&self.0).expect("invalid input id");
        if entry.value.eq(&value) {
          return; // Old value is new value, nothing to do here
        }

        // We don't need to lock here
        // We expect that the Rust borrow checker would only allow one &mut db while no other &db is present
        // We just want a race-free revision counter here to signal "staleness" to later reads
        let new_revision = storage.revision.fetch_add(1, std::sync::atomic::Ordering::Release) + 1;
        let stamped = entry.value_mut();
        stamped.value = value;
        stamped.changed_at = new_revision;
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

          let current_revision = storage.revision.load(std::sync::atomic::Ordering::Acquire);
          #(
            {
              let ingredient = (&*storage.ingredients[start_index + #field_indices] as &dyn std::any::Any)
                .downcast_ref::<typedown_db::InputFieldIngredient<#field_types>>().expect("ingredient type mismatch");
              ingredient.data.insert(id, typedown_db::StampedInputField {
                value: #field_names,
                changed_at: current_revision,
              });
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
  // Try parsing as a function first, then as a struct
  if let Ok(func) = syn::parse::<ItemFn>(item.clone()) {
    return query_derived_fn_impl(func);
  }

  if let Ok(struct_ast) = syn::parse::<ItemStruct>(item.clone()) {
    return query_derived_struct_impl(struct_ast);
  }
  syn::Error::new(
    proc_macro::Span::call_site().into(),
    "#[query_derived] can only be applied to a function or a struct",
  )
  .to_compile_error()
  .into()
}

fn query_derived_fn_impl(func: ItemFn) -> TokenStream {
  let visibility = &func.vis;
  let fn_name = &func.sig.ident;
  let fn_block = &func.block;
  let return_type = match &func.sig.output {
    syn::ReturnType::Type(_, ty) => ty.as_ref(),
    syn::ReturnType::Default => {
      return syn::Error::new_spanned(&func.sig, "derived query must have a return type")
        .to_compile_error()
        .into();
    }
  };

  // Extract arguments: first arg is &db, rest are keys
  let all_args: Vec<_> = func.sig.inputs.iter().collect();
  if all_args.is_empty() {
    return syn::Error::new_spanned(&func.sig, "derived query must take &db as first argument")
      .to_compile_error()
      .into();
  }

  // Skip the first arg (db), collect the rest as key params
  let key_args: Vec<_> = all_args[1..].to_vec();
  let key_names: Vec<_> = key_args
    .iter()
    .filter_map(|arg| {
      if let syn::FnArg::Typed(pat_type) = arg {
        if let syn::Pat::Ident(pat_ident) = pat_type.pat.as_ref() {
          return Some(&pat_ident.ident);
        }
      }
      None
    })
    .collect();
  let key_types: Vec<_> = key_args
    .iter()
    .filter_map(|arg| {
      if let syn::FnArg::Typed(pat_type) = arg {
        return Some(pat_type.ty.as_ref());
      }
      None
    })
    .collect();

  let key_tuple_ty = quote! { (#(#key_types,)*) };

  // The db argument (first arg)
  let db_arg = &all_args[0];

  let mut output: TokenStream = quote! {}.into();

  // Generate marker struct with ingredient index management
  output.extend::<TokenStream>(
    quote! {
      #[allow(non_camel_case_types)]
      #visibility struct #fn_name;

      impl #fn_name {
        fn ingredient_index_lock() -> &'static std::sync::OnceLock<usize> {
          static INDEX: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
          &INDEX
        }

        fn ingredient_index() -> usize {
          *Self::ingredient_index_lock().get()
            .expect("derived query ingredient not registered; was QueryStorage initialized?")
        }

        #[doc(hidden)]
        pub fn set_ingredient_index(index: usize) {
          let _ = Self::ingredient_index_lock().set(index);
        }

        /// The bare query implementation
        fn #fn_name(db: &dyn typedown_db::QueryDatabase, key: #key_tuple_ty) -> #return_type {
          let (#(#key_names,)*) = key;
          #fn_block
        }
      }
    }
    .into(),
  );

  // Register derived query ingredient via inventory
  output.extend::<TokenStream>(
    quote! {
      typedown_db::inventory::submit! {
        typedown_db::Inventory {
          kind: typedown_db::IngredientKind::Derived,
          register: |factories| {
            let index = factories.len();
            factories.push(|| Box::new(
              typedown_db::DerivedQueryIngredient::<#key_tuple_ty, #return_type>::new(index, #fn_name::#fn_name)
            ));
            #fn_name::set_ingredient_index(index);
          },
        }
      }
    }
    .into(),
  );

  // Generate the public wrapper that calls execute_query
  output.extend::<TokenStream>(
    quote! {
      #visibility fn #fn_name(#db_arg, #(#key_names: #key_types),*) -> #return_type {
        let storage = unsafe { db.storage() };
        let ingredient = (&*storage.ingredients[#fn_name::ingredient_index()] as &dyn std::any::Any)
          .downcast_ref::<typedown_db::DerivedQueryIngredient<#key_tuple_ty, #return_type>>()
          .expect("derived ingredient type mismatch");
        ingredient.execute_query(db, (#(#key_names,)*))
      }
    }
    .into(),
  );

  output
}

fn query_derived_struct_impl(struct_ast: ItemStruct) -> TokenStream {
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

  // Validate each field
  for field in &fields {
    let field_ty = &field.ty;
    output.extend::<TokenStream>(
      quote! {
        const _: () = {
          const fn assert_send<T: Send>() {}
          const fn assert_sync<T: Sync>() {}
          const fn assert_clone<T: Clone>() {}
          assert_send::<#field_ty>();
          assert_sync::<#field_ty>();
          assert_clone::<#field_ty>();

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

  // Register per-field ingredients via inventory
  output.extend::<TokenStream>(
    quote! {
      typedown_db::inventory::submit! {
        typedown_db::Inventory {
          kind: typedown_db::IngredientKind::Derived,
          register: |factories| {
            let start_index = factories.len();
            #(
              factories.push(|| Box::new(typedown_db::DerivedFieldIngredient::<#field_types>::new()));
            )*
            #struct_name::set_ingredient_start_index(start_index);
          },
        }
      }
    }
    .into(),
  );

  // Generate getters
  let mut getter_tokens = quote! {};
  for (idx, field) in fields.iter().enumerate() {
    let field_name = field.ident.as_ref().unwrap();
    let field_ty = &field.ty;

    getter_tokens.extend(quote! {
      pub fn #field_name<DB: typedown_db::QueryDatabase>(&self, db: &DB) -> #field_ty {
        let storage = unsafe { db.storage() };
        let ingredient_index = Self::ingredient_start_index() + #idx;
        let ingredient = (&*storage.ingredients[ingredient_index] as &dyn std::any::Any)
          .downcast_ref::<typedown_db::DerivedFieldIngredient<#field_ty>>().expect("ingredient type mismatch");
        let entry = ingredient.data.get(&self.0).expect("invalid derived id");

        // Record dependency if inside a derived query
        storage.with_context(|ctx| {
          if let Some(ctx) = ctx {
            ctx.dependencies.push(typedown_db::Dependency {
              ingredient_index,
              arg_id: self.0,
              changed_at: entry.changed_at,
            });
          }
        });

        entry.value.clone()
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

        /// Create a new derived ID and store its field values
        pub fn new<DB: typedown_db::QueryDatabase>(db: &DB, #(#field_names: #field_types),*) -> Self {
          let storage = unsafe { db.storage() };
          let id = Self::next_id();
          let start_index = Self::ingredient_start_index();
          let current_revision = storage.revision.load(std::sync::atomic::Ordering::Acquire);

          #(
            {
              let ingredient = (&*storage.ingredients[start_index + #field_indices] as &dyn std::any::Any)
                .downcast_ref::<typedown_db::DerivedFieldIngredient<#field_types>>().expect("ingredient type mismatch");
              ingredient.data.insert(id, typedown_db::StampedDerivedField {
                value: #field_names,
                changed_at: current_revision,
              });
            }
          )*

          Self(id)
        }

        #getter_tokens
      }

      impl typedown_db::DerivedId for #struct_name {}

      #[cfg(debug_assertions)]
      const _: () = <#struct_name as typedown_db::DerivedId>::__TYPEDOWN_DERIVED_ID;
    }
    .into(),
  );

  output
}
