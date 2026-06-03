use super::storage::QueryStorage;

pub trait QueryDatabase {
  #[doc(hidden)]
  unsafe fn storage(&self) -> &QueryStorage;

  #[doc(hidden)]
  unsafe fn storage_mut(&mut self) -> &mut QueryStorage;
}
