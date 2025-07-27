//! LazyTower - An O(1) amortized cost alternative to Incremental Merkle Trees
//!
//! This implementation provides efficient proofs with configurable tower width.

pub mod digest;
pub mod error;
pub mod proof;
pub mod tower;

pub use digest::Digest;
pub use error::LazyTowerError;
pub use proof::{MembershipProof, ProofPath, PathElement};
pub use tower::{LazyTower, TowerNode};
