use super::storage::QueryStorage;

pub trait QueryDatabase: std::any::Any {
  #[doc(hidden)]
  unsafe fn storage(&self) -> &QueryStorage;

  #[doc(hidden)]
  unsafe fn storage_mut(&mut self) -> &mut QueryStorage;
}
