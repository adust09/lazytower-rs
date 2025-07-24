//! Tests for error handling

use lazytower_rs::{Digest, LazyTower, LazyTowerError};

/// Mock digest for testing
#[derive(Clone, Debug, PartialEq, Eq)]
struct MockDigest;

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

#[test]
fn test_new_tower_with_invalid_width_zero() {
    let result = LazyTower::<Vec<u8>, MockDigest>::new(0);
    match result {
        Err(LazyTowerError::InvalidWidth { width }) => {
            assert_eq!(width, 0);
        }
        _ => panic!("Expected InvalidWidth error"),
    }
}

#[test]
fn test_new_tower_with_invalid_width_one() {
    let result = LazyTower::<Vec<u8>, MockDigest>::new(1);
    match result {
        Err(LazyTowerError::InvalidWidth { width }) => {
            assert_eq!(width, 1);
        }
        _ => panic!("Expected InvalidWidth error"),
    }
}

#[test]
fn test_new_tower_with_valid_width() {
    let result = LazyTower::<Vec<u8>, MockDigest>::new(4);
    assert!(result.is_ok());

    let tower = result.unwrap();
    assert_eq!(tower.width(), 4);
}

#[test]
fn test_generate_proof_invalid_index() {
    let tower = LazyTower::<Vec<u8>, MockDigest>::new(4).unwrap();
    let result = tower.generate_proof(0);

    match result {
        Err(LazyTowerError::InvalidIndex { index, max }) => {
            assert_eq!(index, 0);
            assert_eq!(max, 0);
        }
        _ => panic!("Expected InvalidIndex error for empty tower"),
    }
}

#[test]
fn test_generate_proof_index_out_of_bounds() {
    let mut tower = LazyTower::<Vec<u8>, MockDigest>::new(4).unwrap();
    tower.append(vec![1, 2, 3]);
    tower.append(vec![4, 5, 6]);

    let result = tower.generate_proof(5);

    match result {
        Err(LazyTowerError::InvalidIndex { index, max }) => {
            assert_eq!(index, 5);
            assert_eq!(max, 2);
        }
        _ => panic!("Expected InvalidIndex error for out of bounds index"),
    }
}
