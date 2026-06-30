# Stable Hasher

This module implements session-independent and architecture-independent hashing/sorting.

It draws inspiration from `rustc`, check my related docs here:

- [dboxide findings of `rustc`](https://huydo862003.github.io/dboxide/research/resources/rustc/SUMMARY.html)
- [`compiler/rustc_data_structures/src/stable_hash.rs`](https://github.com/rust-lang/rust/blob/63f05e3635171e7ac3f9ca78bad6c71052cda5a3/compiler/rustc_data_structures/src/stable_hash.rs) - trait definitions

## High-Level Flow

1. The consumer defines a `StableHashCtx` that returns a stable representation of a session-dependent identifier.
2. To stable hash a value, create a `StableHasher`, then pass the hash context and hasher to the `.stable_hash()` method.
   a. The hashing algorithm may use the `StableCompare` trait for session-independent stable sorting.
