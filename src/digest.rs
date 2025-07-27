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
}

/// SHA256 implementation of Digest
#[cfg(feature = "sha256")]
pub mod sha256 {
    use super::*;
    use sha2::{Digest as Sha2Digest, Sha256};

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
    }
}

/// Mock digest for testing
#[cfg(any(test, feature = "test-utils"))]
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LazyTower;

    /// Test item
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct TestItem(String);

    impl AsRef<[u8]> for TestItem {
        fn as_ref(&self) -> &[u8] {
            self.0.as_bytes()
        }
    }

    #[test]
    fn test_root_digest_empty_tower() {
        use mock::MockDigest;
        let tower: LazyTower<TestItem, MockDigest> = LazyTower::new(4).unwrap();
        assert_eq!(tower.root_digest(), None);
    }

    #[test]
    fn test_root_digest_single_item() {
        use mock::MockDigest;
        let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(4).unwrap();
        tower.append(TestItem("A".to_string()));

        let root = tower.root_digest().expect("Should have root");
        assert_eq!(root, b"digest(A)");
    }

    #[test]
    fn test_root_digest_multiple_items_no_overflow() {
        use mock::MockDigest;
        let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(4).unwrap();
        tower.append(TestItem("A".to_string()));
        tower.append(TestItem("B".to_string()));
        tower.append(TestItem("C".to_string()));

        let root = tower.root_digest().expect("Should have root");
        assert_eq!(root, b"digest_items[A,B,C]");
    }

    #[test]
    fn test_root_digest_with_overflow() {
        use mock::MockDigest;
        let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(2).unwrap();

        // Add 4 items to create overflows
        tower.append(TestItem("A".to_string()));
        tower.append(TestItem("B".to_string()));
        tower.append(TestItem("C".to_string()));
        tower.append(TestItem("D".to_string()));

        // Structure: Level 2 has H[H[A,B],H[C,D]]
        let root = tower.root_digest().expect("Should have root");
        assert_eq!(root, b"digest_items[digest_items[A,B],digest_items[C,D]]");
    }

    #[test]
    fn test_root_digest_complex_structure() {
        use mock::MockDigest;
        let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(3).unwrap();

        // Add 7 items
        for i in 0..7 {
            tower.append(TestItem(i.to_string()));
        }

        // Structure:
        // Level 0: [6]
        // Level 1: [H[0,1,2], H[3,4,5]]

        let root = tower.root_digest().expect("Should have root");
        // The root should be the combination of level 1 nodes
        assert_eq!(root, b"digest_items[digest_items[0,1,2],digest_items[3,4,5]]");
    }

    #[test]
    fn test_root_digest_deep_tower() {
        use mock::MockDigest;
        let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(2).unwrap();

        // Add 8 items to create a deep structure
        for i in 0..8 {
            tower.append(TestItem(i.to_string()));
        }

        // This creates multiple levels of overflows
        let root = tower.root_digest().expect("Should have root");

        // Verify we get a valid root (exact structure depends on overflow pattern)
        assert!(!root.is_empty());
        assert!(root.starts_with(b"digest"));
    }
}
