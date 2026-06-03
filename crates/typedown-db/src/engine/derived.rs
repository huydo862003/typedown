//! Derived query engine for the incremental database

use std::marker::PhantomData;

/// A trait for a derived state
/// Required to be Clone, Send and Sync
pub trait Derived: Clone + Send + Sync {}

/// A fast id for a derived state
/// Input id is bound to a database's lifetime
pub struct DerivedId<'db, T: Derived>(usize, PhantomData<T>, PhantomData<&'db ()>);
