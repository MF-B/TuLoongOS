//! Synchronization and interior mutability primitives

mod up;
pub mod mutex;
pub mod semaphore;
pub mod condvar;

pub use up::UPSafeCell;
