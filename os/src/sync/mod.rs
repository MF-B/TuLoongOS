//! Synchronization and interior mutability primitives

mod up;
pub mod mutex;
pub mod semaphore;

pub use up::UPSafeCell;
