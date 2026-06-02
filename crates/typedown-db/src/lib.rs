//! A salsa-like database for incremental Typedown compilation
//! Salsa is not used so I can fully control the incrementalism + task-based parallelism
pub mod derived;
pub mod inputs;

pub struct Database {}
