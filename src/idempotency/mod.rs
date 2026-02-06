//! Idempotency store for preventing duplicate write operations
//!
//! Provides local storage of operation results with:
//! - TTL-based expiration (7 days default)
//! - Capacity limits (10,000 entries default)
//! - Automatic garbage collection
//! - Request fingerprinting for duplicate detection

pub mod handler;
pub mod store;
pub mod types;

pub use handler::{IdempotencyCheckResult, IdempotencyHandler};
pub use store::{IdempotencyError, IdempotencyStore};
pub use types::{IdempotencyEntry, IdempotencyStatus, RequestFingerprint, ScopedKey};
