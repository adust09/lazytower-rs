//! LazyTower - An O(1) amortized cost alternative to Incremental Merkle Trees
//!
//! This implementation provides efficient membership proofs with configurable tower width.

pub mod digest;
pub mod proof;
pub mod tower;

pub use digest::Digest;
pub use proof::{MembershipProof, ProofPath};
pub use tower::{LazyTower, TowerNode};

#[cfg(test)]
mod tests {
    // Basic tests can be added here
}
