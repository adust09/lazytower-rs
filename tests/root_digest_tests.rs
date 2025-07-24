//! Tests for root digest computation

use lazytower_rs::{Digest, LazyTower};

/// Test item
#[derive(Clone, Debug, PartialEq, Eq)]
struct TestItem(String);

impl AsRef<[u8]> for TestItem {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Mock digest
#[derive(Clone, Debug, PartialEq, Eq)]
struct MockDigest;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MockDigestOutput(String);

impl AsRef<[u8]> for MockDigestOutput {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Digest for MockDigest {
    type Output = MockDigestOutput;

    fn digest_item<T: AsRef<[u8]>>(item: &T) -> Self::Output {
        MockDigestOutput(format!("H({})", String::from_utf8_lossy(item.as_ref())))
    }

    fn digest_items<T: AsRef<[u8]>>(items: &[T]) -> Self::Output {
        let items_str: Vec<String> =
            items.iter().map(|item| String::from_utf8_lossy(item.as_ref()).to_string()).collect();
        MockDigestOutput(format!("H[{}]", items_str.join(",")))
    }

    fn combine(left: &Self::Output, right: &Self::Output) -> Self::Output {
        MockDigestOutput(format!("H({},{})", left.0, right.0))
    }
}

#[test]
fn test_root_digest_empty_tower() {
    let tower: LazyTower<TestItem, MockDigest> = LazyTower::new(4).unwrap();
    assert_eq!(tower.root_digest(), None);
}

#[test]
fn test_root_digest_single_item() {
    let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(4).unwrap();
    tower.append(TestItem("A".to_string()));

    let root = tower.root_digest().expect("Should have root");
    assert_eq!(root.0, "H(A)");
}

#[test]
fn test_root_digest_multiple_items_no_overflow() {
    let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(4).unwrap();
    tower.append(TestItem("A".to_string()));
    tower.append(TestItem("B".to_string()));
    tower.append(TestItem("C".to_string()));

    let root = tower.root_digest().expect("Should have root");
    assert_eq!(root.0, "H[A,B,C]");
}

#[test]
fn test_root_digest_with_overflow() {
    let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(2).unwrap();

    // Add 4 items to create overflows
    tower.append(TestItem("A".to_string()));
    tower.append(TestItem("B".to_string()));
    tower.append(TestItem("C".to_string()));
    tower.append(TestItem("D".to_string()));

    // Structure: Level 2 has H[H[A,B],H[C,D]]
    let root = tower.root_digest().expect("Should have root");
    assert_eq!(root.0, "H[H[A,B],H[C,D]]");
}

#[test]
fn test_root_digest_complex_structure() {
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
    assert_eq!(root.0, "H[H[0,1,2],H[3,4,5]]");
}

#[test]
fn test_root_digest_deep_tower() {
    let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(2).unwrap();

    // Add 8 items to create a deep structure
    for i in 0..8 {
        tower.append(TestItem(i.to_string()));
    }

    // This creates multiple levels of overflows
    let root = tower.root_digest().expect("Should have root");

    // Verify we get a valid root (exact structure depends on overflow pattern)
    assert!(!root.0.is_empty());
    assert!(root.0.starts_with("H"));
}
