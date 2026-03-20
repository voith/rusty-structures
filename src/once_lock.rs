//! A minimal `OnceLock<T>` implementation for learning purposes.
//!
//! What it does:
//! - Stores a value that is initialized at most once.
//! - Lets many callers race to read the same value, while only one caller
//!   actually runs the initializer closure.
//! - Returns a shared `&T` after initialization completes.
//!
//! How it works:
//! - `state` is a small atomic state machine:
//!   - `UNINIT` means no value has been produced yet.
//!   - `INITIALIZING` means one thread won the race and is currently running
//!     the initializer.
//!   - `INITIALIZED` means the value is stored and can be read by everyone.
//! - The winning thread changes the state from `UNINIT` to `INITIALIZING`,
//!   computes the value, writes it into `value`, then publishes success with a
//!   `Release` store to `INITIALIZED`.
//! - Other threads spin until they observe `INITIALIZED`, then read the same
//!   shared value using an `Acquire` load.
//! - If the initializer panics, a small drop guard resets the state back to
//!   `UNINIT` so another attempt can be made later.
//!
//! Example:
//! ```
//! use data_structures::once_lock::OnceLock;
//!
//! static CONFIG: OnceLock<String> = OnceLock::new();
//!
//! let value = CONFIG.get_or_init(|| "ready".to_string());
//! assert_eq!(value, "ready");
//!
//! // Later calls reuse the first value and do not rerun the initializer.
//! let same = CONFIG.get_or_init(|| "different".to_string());
//! assert_eq!(same, "ready");
//! ```
//!
//! Current problems and limitations:
//! - This is not a production-ready replacement for `std::sync::OnceLock`.
//! - The retry path currently uses recursion when initialization previously
//!   failed, which can grow the stack if retries keep happening.
//! - Waiting threads use a busy spin with `thread::yield_now()`, which is
//!   simple but inefficient under contention.
//! - Re-entrant initialization on the same `OnceLock` is not handled
//!   explicitly and may deadlock or behave poorly.
//! - The unsafe trait and raw-pointer sections need especially careful review
//!   because the correctness of the whole type depends on them.
//!
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};

const UNINIT: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

pub struct OnceLock<T> {
    state: AtomicUsize,
    value: UnsafeCell<Option<T>>,
}

unsafe impl<T: Send + Sync> Sync for OnceLock<T> {}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicUsize::new(UNINIT),
            value: UnsafeCell::new(None),
        }
    }

    pub fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        let mut init = Some(init);

        loop {
            if self.state.load(Ordering::Acquire) == INITIALIZED {
                return self.get_unchecked();
            }

            if self
                .state
                .compare_exchange(UNINIT, INITIALIZING, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                // Guard ensures we reset state if panic happens
                struct Guard<'a> {
                    state: &'a AtomicUsize,
                    active: bool,
                }

                impl Drop for Guard<'_> {
                    fn drop(&mut self) {
                        if self.active {
                            // Initialization failed → reset
                            self.state.store(UNINIT, Ordering::Release);
                        }
                    }
                }

                let mut guard = Guard {
                    state: &self.state,
                    active: true,
                };

                // Run initializer (may panic)
                let value = init.take().unwrap()();

                unsafe {
                    *self.value.get() = Some(value);
                }

                // Mark success
                self.state.store(INITIALIZED, Ordering::Release);

                // Disable rollback
                guard.active = false;

                return self.get_unchecked();
            }

            while self.state.load(Ordering::Acquire) == INITIALIZING {
                std::thread::yield_now();
            }
        }
    }

    fn get_unchecked(&self) -> &T {
        unsafe {
            (*self.value.get()).as_ref().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn new_starts_uninitialized() {
        let lock = OnceLock::<usize>::new();

        assert_eq!(lock.state.load(Ordering::Acquire), UNINIT);
        assert!(unsafe { (*lock.value.get()).is_none() });
    }

    #[test]
    fn get_or_init_stores_and_reuses_the_first_value() {
        let lock = OnceLock::new();
        let first = lock.get_or_init(|| String::from("hello"));
        let second = lock.get_or_init(|| String::from("goodbye"));

        assert_eq!(first, "hello");
        assert!(std::ptr::eq(first, second));
        assert_eq!(lock.state.load(Ordering::Acquire), INITIALIZED);
    }

    #[test]
    fn initializer_runs_only_once() {
        let lock = OnceLock::new();
        let calls = AtomicUsize::new(0);

        let first = lock.get_or_init(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            42
        });
        let second = lock.get_or_init(|| {
            calls.fetch_add(1, Ordering::SeqCst);
            99
        });

        assert_eq!(*first, 42);
        assert_eq!(*second, 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn panic_during_initialization_resets_the_lock() {
        let lock = OnceLock::new();

        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            lock.get_or_init(|| -> usize { panic!("boom") });
        }));

        assert!(result.is_err());
        assert_eq!(lock.state.load(Ordering::Acquire), UNINIT);
        assert!(unsafe { (*lock.value.get()).is_none() });

        let value = lock.get_or_init(|| 7);
        assert_eq!(*value, 7);
        assert_eq!(lock.state.load(Ordering::Acquire), INITIALIZED);
    }

    #[test]
    fn concurrent_callers_share_one_initialization() {
        let lock = Arc::new(OnceLock::new());
        let started = Arc::new(AtomicUsize::new(0));
        let finished = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..8 {
            let lock = Arc::clone(&lock);
            let started = Arc::clone(&started);
            let finished = Arc::clone(&finished);
            handles.push(thread::spawn(move || {
                let value = lock.get_or_init(|| {
                    started.fetch_add(1, Ordering::SeqCst);
                    thread::sleep(std::time::Duration::from_millis(20));
                    finished.fetch_add(1, Ordering::SeqCst);
                    1234
                });
                *value
            }));
        }

        for handle in handles {
            assert_eq!(handle.join().unwrap(), 1234);
        }

        assert_eq!(started.load(Ordering::SeqCst), 1);
        assert_eq!(finished.load(Ordering::SeqCst), 1);
        assert_eq!(*lock.get_or_init(|| 9999), 1234);
    }

    #[test]
    fn another_thread_can_retry_after_a_panicking_initializer() {
        let lock = Arc::new(OnceLock::new());
        let attempts = Arc::new(AtomicUsize::new(0));

        let first_lock = Arc::clone(&lock);
        let first_attempts = Arc::clone(&attempts);
        let first = thread::spawn(move || {
            panic::catch_unwind(AssertUnwindSafe(|| {
                first_lock.get_or_init(|| -> usize {
                    first_attempts.fetch_add(1, Ordering::SeqCst);
                    panic!("first init fails");
                });
            }))
        });

        assert!(first.join().unwrap().is_err());

        let second_lock = Arc::clone(&lock);
        let second_attempts = Arc::clone(&attempts);
        let second = thread::spawn(move || {
            *second_lock.get_or_init(|| {
                second_attempts.fetch_add(1, Ordering::SeqCst);
                55
            })
        });

        assert_eq!(second.join().unwrap(), 55);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(lock.state.load(Ordering::Acquire), INITIALIZED);
    }
}
