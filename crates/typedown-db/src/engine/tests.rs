#[cfg(test)]
mod derived_query_tests {
  use std::sync::atomic::Ordering;

  use typedown_macros::{query_db, query_derived, query_input};

  use crate::{QueryDatabase, QueryStorage};

  #[query_db]
  struct Database {
    storage: QueryStorage,
  }

  #[query_input]
  struct FibInput {
    n: usize,
  }

  #[query_derived]
  struct FibResult {
    #[id]
    n: usize,
    value: usize,
  }

  #[query_derived]
  fn fibonacci(db: &Database, input: FibInput) -> FibResult {
    let n = input.n(db);
    if n <= 1 {
      return FibResult::new(db, n, n);
    }

    let input_a = FibInput::new(db, n - 1);
    let input_b = FibInput::new(db, n - 2);

    let a = fibonacci(db, input_a);
    let b = fibonacci(db, input_b);

    FibResult::new(db, n, a.value(db) + b.value(db))
  }

  #[test]
  fn fibonacci_base_cases() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let input0 = FibInput::new(&db, 0);
    let input1 = FibInput::new(&db, 1);

    let result0 = fibonacci(&db, input0);
    let result1 = fibonacci(&db, input1);

    assert_eq!(result0.value(&db), 0);
    assert_eq!(result1.value(&db), 1);
  }

  #[test]
  fn fibonacci_recursive() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let input = FibInput::new(&db, 10);
    let result = fibonacci(&db, input);

    assert_eq!(result.value(&db), 55);
    assert_eq!(result.n(&db), 10);
  }

  #[test]
  fn fibonacci_cached_rerun() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let input = FibInput::new(&db, 10);

    let result1 = fibonacci(&db, input);
    let result2 = fibonacci(&db, input);

    assert_eq!(result1, result2);
    assert_eq!(result1.value(&db), 55);
  }

  #[test]
  fn fibonacci_does_not_bump_revision() {
    let db = Database {
      storage: QueryStorage::default(),
    };

    let input = FibInput::new(&db, 10);

    let rev_before = db.storage.revision.load(Ordering::Acquire);
    let _result = fibonacci(&db, input);
    let rev_after = db.storage.revision.load(Ordering::Acquire);

    assert_eq!(
      rev_before, rev_after,
      "revision should not bump from derived query execution"
    );
  }
}
