//! LazyTower - An O(1) amortized cost alternative to Incremental Merkle Trees
//! 
//! This implementation provides efficient membership proofs with configurable tower width.

pub mod tower;
pub mod digest;
pub mod proof;

pub use tower::{LazyTower, TowerNode};
pub use digest::Digest;
pub use proof::{MembershipProof, ProofPath};

#[cfg(test)]
mod tests {
    // Basic tests can be added here
}