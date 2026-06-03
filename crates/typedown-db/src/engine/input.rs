//! Input declaration engine for the incremental database

use std::marker::PhantomData;

/// A trait for an input state
/// Required to be Clone, Send and Sync
pub trait Input: Clone + Send + Sync {}

/// A fast id for an input state
/// Input id is bound to a database's lifetime
pub struct InputId<'db, T: Input>(usize, PhantomData<T>, PhantomData<&'db ()>);
