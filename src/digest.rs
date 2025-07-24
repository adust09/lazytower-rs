//! Digest trait for hash function abstraction

use std::fmt::Debug;

/// Trait for digest/hash functions used in LazyTower
pub trait Digest: Clone + Debug + PartialEq + Eq {
    /// The output type of the digest function
    type Output: Clone + Debug + PartialEq + Eq + AsRef<[u8]>;
    
    /// Compute the digest of a single item
    fn digest_item<T: AsRef<[u8]>>(item: &T) -> Self::Output;
    
    /// Compute the digest of multiple items (for level computation)
    fn digest_items<T: AsRef<[u8]>>(items: &[T]) -> Self::Output;
    
    /// Combine two digests (for Merkle tree construction)
    fn combine(left: &Self::Output, right: &Self::Output) -> Self::Output;
}

/// SHA256 implementation of Digest
#[cfg(feature = "sha256")]
pub mod sha256 {
    use super::*;
    use sha2::{Sha256, Digest as Sha2Digest};
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Sha256Digest;
    
    impl Digest for Sha256Digest {
        type Output = [u8; 32];
        
        fn digest_item<T: AsRef<[u8]>>(item: &T) -> Self::Output {
            let mut hasher = Sha256::new();
            hasher.update(item.as_ref());
            hasher.finalize().into()
        }
        
        fn digest_items<T: AsRef<[u8]>>(items: &[T]) -> Self::Output {
            let mut hasher = Sha256::new();
            for item in items {
                hasher.update(item.as_ref());
            }
            hasher.finalize().into()
        }
        
        fn combine(left: &Self::Output, right: &Self::Output) -> Self::Output {
            let mut hasher = Sha256::new();
            hasher.update(left);
            hasher.update(right);
            hasher.finalize().into()
        }
    }
}

/// Mock digest for testing
#[cfg(test)]
pub mod mock {
    use super::*;
    
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct MockDigest;
    
    impl Digest for MockDigest {
        type Output = Vec<u8>;
        
        fn digest_item<T: AsRef<[u8]>>(item: &T) -> Self::Output {
            let mut result = b"digest(".to_vec();
            result.extend_from_slice(item.as_ref());
            result.extend_from_slice(b")");
            result
        }
        
        fn digest_items<T: AsRef<[u8]>>(items: &[T]) -> Self::Output {
            let mut result = b"digest_items[".to_vec();
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.extend_from_slice(b",");
                }
                result.extend_from_slice(item.as_ref());
            }
            result.extend_from_slice(b"]");
            result
        }
        
        fn combine(left: &Self::Output, right: &Self::Output) -> Self::Output {
            let mut result = b"combine(".to_vec();
            result.extend_from_slice(left);
            result.extend_from_slice(b",");
            result.extend_from_slice(right);
            result.extend_from_slice(b")");
            result
        }
    }
}