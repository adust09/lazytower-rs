//! Tests for membership proof generation and verification

use lazytower_rs::{Digest, LazyTower, MembershipProof, ProofPath};

/// Test item that can be converted to bytes
#[derive(Clone, Debug, PartialEq, Eq)]
struct TestItem(String);

impl AsRef<[u8]> for TestItem {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// Mock digest for testing
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
fn test_proof_path_verification_single_item() {
    let item = TestItem("data".to_string());
    let expected_root = MockDigest::digest_item(&item);

    let path = ProofPath::<MockDigest>::new();

    // Single item proof - no siblings
    assert!(path.verify(&item, &expected_root));
}

#[test]
fn test_proof_path_verification_with_siblings() {
    let item = TestItem("A".to_string());
    let sibling = TestItem("B".to_string());

    let item_digest = MockDigest::digest_item(&item);
    let sibling_digest = MockDigest::digest_item(&sibling);
    let expected_root = MockDigest::combine(&item_digest, &sibling_digest);

    let mut path = ProofPath::<MockDigest>::new();
    path.add_right(sibling_digest);

    assert!(path.verify(&item, &expected_root));
}

#[test]
fn test_proof_path_verification_multi_level() {
    let item = TestItem("A".to_string());

    // Build a multi-level tree structure
    // Level 0: A, B, C, D
    // Level 1: H(A,B), H(C,D)
    // Root: H(H(A,B), H(C,D))

    let b_digest = MockDigest::digest_item(&TestItem("B".to_string()));
    let cd_digest = MockDigest::combine(
        &MockDigest::digest_item(&TestItem("C".to_string())),
        &MockDigest::digest_item(&TestItem("D".to_string())),
    );

    let ab_digest = MockDigest::combine(&MockDigest::digest_item(&item), &b_digest);
    let root = MockDigest::combine(&ab_digest, &cd_digest);

    let mut path = ProofPath::<MockDigest>::new();
    path.add_right(b_digest);
    path.add_right(cd_digest);

    assert!(path.verify(&item, &root));
}

#[test]
fn test_proof_path_verification_fails_wrong_root() {
    let item = TestItem("A".to_string());
    let wrong_root = MockDigest::digest_item(&TestItem("wrong".to_string()));

    let path = ProofPath::<MockDigest>::new();

    assert!(!path.verify(&item, &wrong_root));
}

#[test]
fn test_proof_path_verification_fails_wrong_sibling() {
    let item = TestItem("A".to_string());
    let correct_sibling = TestItem("B".to_string());
    let wrong_sibling = TestItem("C".to_string());

    let item_digest = MockDigest::digest_item(&item);
    let correct_sibling_digest = MockDigest::digest_item(&correct_sibling);
    let expected_root = MockDigest::combine(&item_digest, &correct_sibling_digest);

    let mut path = ProofPath::<MockDigest>::new();
    path.add_right(MockDigest::digest_item(&wrong_sibling));

    assert!(!path.verify(&item, &expected_root));
}

#[test]
fn test_membership_proof_complete() {
    let item = TestItem("data".to_string());
    let sibling = TestItem("sibling".to_string());

    let item_digest = MockDigest::digest_item(&item);
    let sibling_digest = MockDigest::digest_item(&sibling);
    let root = MockDigest::combine(&item_digest, &sibling_digest);

    let mut path = ProofPath::<MockDigest>::new();
    path.add_right(sibling_digest);

    let proof = MembershipProof { item: item.clone(), path, root };

    assert!(proof.verify());
}

// Integration test with LazyTower
#[test]
fn test_generate_proof_for_tower_item() {
    let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(2).unwrap();

    // Add 4 items to create a specific structure
    tower.append(TestItem("A".to_string()));
    tower.append(TestItem("B".to_string()));
    tower.append(TestItem("C".to_string()));
    tower.append(TestItem("D".to_string()));

    // After this, we should have:
    // Level 0: empty
    // Level 1: empty
    // Level 2: H[H[A,B],H[C,D]]

    // TODO: Once we implement proof generation in the tower,
    // we'll test generating a proof for item "A"
    // The proof path should be:
    // 1. Right sibling: B (at original level)
    // 2. Right sibling: H[C,D] (at level 1)

    // For now, just verify the structure
    assert_eq!(tower.height(), 3);
    assert_eq!(tower.len(), 4);
}

#[test]
fn test_proof_path_left_and_right_siblings() {
    let item = TestItem("B".to_string());

    // Tree structure where B has both left and right siblings
    // A, B, C at level 0
    let a_digest = MockDigest::digest_item(&TestItem("A".to_string()));
    let b_digest = MockDigest::digest_item(&item);
    let c_digest = MockDigest::digest_item(&TestItem("C".to_string()));

    // Compute root as H(A, H(B, C))
    let bc_digest = MockDigest::combine(&b_digest, &c_digest);
    let root = MockDigest::combine(&a_digest, &bc_digest);

    let mut path = ProofPath::<MockDigest>::new();
    path.add_right(c_digest); // First combine B with C
    path.add_left(a_digest); // Then combine A with H(B,C)

    assert!(path.verify(&item, &root));
}

// TODO: Add tests for actual proof generation once implemented in LazyTower
// #[test]
// fn test_tower_generate_membership_proof() {
//     let mut tower: LazyTower<TestItem, MockDigest> = LazyTower::new(3).unwrap();
//
//     // Add items
//     let items = vec!["A", "B", "C", "D", "E"];
//     for item in &items {
//         tower.append(TestItem(item.to_string()));
//     }
//
//     // Generate proof for item "B"
//     let proof = tower.generate_proof(1).expect("Should generate proof");
//
//     // Verify the proof
//     assert!(proof.verify());
//     assert_eq!(proof.item.0, "B");
// }
