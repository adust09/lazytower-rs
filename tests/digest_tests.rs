//! Tests for digest computation

use lazytower_rs::{Digest, LazyTower, TowerNode};

/// Test item that can be converted to bytes
#[derive(Clone, Debug, PartialEq, Eq)]
struct TestItem(String);

impl AsRef<[u8]> for TestItem {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Mock digest for testing that tracks operations
#[derive(Clone, Debug, PartialEq, Eq)]
struct TrackedDigest;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TrackedDigestOutput(String);

impl AsRef<[u8]> for TrackedDigestOutput {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Digest for TrackedDigest {
    type Output = TrackedDigestOutput;

    fn digest_item<T: AsRef<[u8]>>(item: &T) -> Self::Output {
        TrackedDigestOutput(format!("D({})", String::from_utf8_lossy(item.as_ref())))
    }

    fn digest_items<T: AsRef<[u8]>>(items: &[T]) -> Self::Output {
        let items_str: Vec<String> =
            items.iter().map(|item| String::from_utf8_lossy(item.as_ref()).to_string()).collect();
        TrackedDigestOutput(format!("D[{}]", items_str.join(",")))
    }

    fn combine(left: &Self::Output, right: &Self::Output) -> Self::Output {
        TrackedDigestOutput(format!("C({},{})", left.0, right.0))
    }
}

#[test]
fn test_digest_computation_on_overflow() {
    let mut tower: LazyTower<TestItem, TrackedDigest> = LazyTower::new(2).unwrap();

    // Add two items to trigger overflow
    tower.append(TestItem("A".to_string()));
    tower.append(TestItem("B".to_string()));

    // Level 0 should be empty after overflow
    assert_eq!(tower.level(0).unwrap().len(), 0);

    // Level 1 should contain the digest of A and B
    assert_eq!(tower.height(), 2);
    let level1 = tower.level(1).unwrap();
    assert_eq!(level1.len(), 1);

    match &level1[0] {
        TowerNode::Digest(digest) => {
            // The digest should be D[A,B] based on our TrackedDigest implementation
            assert_eq!(digest.0, "D[A,B]");
        }
        _ => panic!("Expected digest node at level 1"),
    }
}

#[test]
fn test_nested_digest_computation() {
    let mut tower: LazyTower<TestItem, TrackedDigest> = LazyTower::new(2).unwrap();

    // Add 4 items to trigger two overflows and then a nested overflow
    tower.append(TestItem("A".to_string()));
    tower.append(TestItem("B".to_string()));
    tower.append(TestItem("C".to_string()));
    tower.append(TestItem("D".to_string()));

    // After 4 items with width 2:
    // - A,B overflow to level 1 as D[A,B]
    // - C,D overflow to level 1 as D[C,D]
    // - Level 1 overflows to level 2 as D[D[A,B],D[C,D]]

    assert_eq!(tower.height(), 3);
    assert_eq!(tower.level(0).unwrap().len(), 0);
    assert_eq!(tower.level(1).unwrap().len(), 0);

    let level2 = tower.level(2).unwrap();
    assert_eq!(level2.len(), 1);

    match &level2[0] {
        TowerNode::Digest(digest) => {
            assert_eq!(digest.0, "D[D[A,B],D[C,D]]");
        }
        _ => panic!("Expected digest node at level 2"),
    }
}

#[test]
fn test_mixed_nodes_and_digests() {
    let mut tower: LazyTower<TestItem, TrackedDigest> = LazyTower::new(3).unwrap();

    // Add 7 items with width 3
    for i in 0..7 {
        tower.append(TestItem(i.to_string()));
    }

    // After 7 items with width 3:
    // - Items 0,1,2 overflow to level 1 as D[0,1,2]
    // - Items 3,4,5 overflow to level 1 as D[3,4,5]
    // - Item 6 remains at level 0

    assert_eq!(tower.len(), 7);
    assert_eq!(tower.level(0).unwrap().len(), 1);

    // Check level 0 has item 6
    match &tower.level(0).unwrap()[0] {
        TowerNode::Item(item) => assert_eq!(item.0, "6"),
        _ => panic!("Expected item node at level 0"),
    }

    // Check level 1 has two digests
    let level1 = tower.level(1).unwrap();
    assert_eq!(level1.len(), 2);

    match &level1[0] {
        TowerNode::Digest(digest) => assert_eq!(digest.0, "D[0,1,2]"),
        _ => panic!("Expected digest node at level 1 position 0"),
    }

    match &level1[1] {
        TowerNode::Digest(digest) => assert_eq!(digest.0, "D[3,4,5]"),
        _ => panic!("Expected digest node at level 1 position 1"),
    }
}

#[test]
fn test_digest_of_mixed_nodes() {
    let mut tower: LazyTower<TestItem, TrackedDigest> = LazyTower::new(2).unwrap();

    // This test ensures that digest_items works correctly on TowerNode types
    // which can be either Item or Digest variants

    // Add 6 items to create a complex structure
    for i in 0..6 {
        tower.append(TestItem(i.to_string()));
    }

    // After 6 items with width 2:
    // - 0,1 -> D[0,1] at level 1
    // - 2,3 -> D[2,3] at level 1
    // - D[0,1],D[2,3] -> D[D[0,1],D[2,3]] at level 2
    // - 4,5 -> D[4,5] at level 1

    assert_eq!(tower.height(), 3);

    // Level 1 should have one digest (D[4,5])
    let level1 = tower.level(1).unwrap();
    assert_eq!(level1.len(), 1);
    match &level1[0] {
        TowerNode::Digest(digest) => assert_eq!(digest.0, "D[4,5]"),
        _ => panic!("Expected digest at level 1"),
    }

    // Level 2 should have the nested digest
    let level2 = tower.level(2).unwrap();
    assert_eq!(level2.len(), 1);
    match &level2[0] {
        TowerNode::Digest(digest) => assert_eq!(digest.0, "D[D[0,1],D[2,3]]"),
        _ => panic!("Expected digest at level 2"),
    }
}

#[test]
fn test_digest_trait_implementation() {
    // Test the digest trait methods directly
    let item = TestItem("hello".to_string());
    let digest = TrackedDigest::digest_item(&item);
    assert_eq!(digest.0, "D(hello)");

    let items =
        vec![TestItem("a".to_string()), TestItem("b".to_string()), TestItem("c".to_string())];
    let multi_digest = TrackedDigest::digest_items(&items);
    assert_eq!(multi_digest.0, "D[a,b,c]");

    let left = TrackedDigestOutput("left".to_string());
    let right = TrackedDigestOutput("right".to_string());
    let combined = TrackedDigest::combine(&left, &right);
    assert_eq!(combined.0, "C(left,right)");
}

#[cfg(feature = "sha256")]
mod sha256_tests {
    use super::*;
    use lazytower_rs::digest::sha256::Sha256Digest;

    #[test]
    fn test_sha256_digest_computation() {
        let mut tower: LazyTower<TestItem, Sha256Digest> = LazyTower::new(2).unwrap();

        // Add items and verify digests are computed
        tower.append(TestItem("hello".to_string()));
        tower.append(TestItem("world".to_string()));

        // After overflow, level 1 should have a SHA256 digest
        assert_eq!(tower.level(0).unwrap().len(), 0);
        assert_eq!(tower.level(1).unwrap().len(), 1);

        match &tower.level(1).unwrap()[0] {
            TowerNode::Digest(digest) => {
                // Verify it's a valid 32-byte SHA256 hash
                assert_eq!(digest.len(), 32);
            }
            _ => panic!("Expected digest node at level 1"),
        }
    }
}
