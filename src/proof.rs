//! proof structures and generation

use crate::digest::Digest;

/// A path element in a proof
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathElement<D: Digest> {
    /// Siblings at the same level with position information
    Siblings {
        /// Position of the item/node being proved (0-indexed)
        position: usize,
        /// Digests of sibling nodes (excluding self)
        siblings: Vec<D::Output>,
    },
    /// Raw siblings for level 0 (stores raw bytes to match root computation)
    RawSiblings {
        /// Position of the item being proved (0-indexed)
        position: usize,
        /// Raw bytes of sibling items (excluding self)
        siblings: Vec<Vec<u8>>,
    },
}

/// A proof path from item to root
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofPath<D: Digest> {
    /// The path elements from bottom to top
    pub elements: Vec<PathElement<D>>,
}

/// A complete proof
#[derive(Debug, Clone)]
pub struct MembershipProof<T, D: Digest> {
    /// The item being proved
    pub item: T,
    /// The proof path
    pub path: ProofPath<D>,
    /// The root digest
    pub root: D::Output,
}

impl<D: Digest> ProofPath<D> {
    /// Create a new empty proof path
    pub fn new() -> Self {
        Self { elements: Vec::new() }
    }

    /// Add siblings to the path
    pub fn add_siblings(&mut self, position: usize, siblings: Vec<D::Output>) {
        self.elements.push(PathElement::Siblings { position, siblings });
    }

    /// Add raw siblings to the path (for level 0)
    pub fn add_raw_siblings(&mut self, position: usize, siblings: Vec<Vec<u8>>) {
        self.elements.push(PathElement::RawSiblings { position, siblings });
    }

    /// Verify a proof path for an item
    pub fn verify<T: AsRef<[u8]>>(&self, item: &T, expected_root: &D::Output) -> bool {
        // Start with the raw item for the first level
        let mut current_is_raw = true;
        let current_raw: Option<Vec<u8>> = Some(item.as_ref().to_vec());
        let mut current_digest: Option<D::Output> = None;

        for (level_idx, element) in self.elements.iter().enumerate() {
            match element {
                PathElement::Siblings { position, siblings } => {
                    // Get current value as digest
                    let current = if current_is_raw {
                        D::digest_item(&current_raw.as_ref().unwrap())
                    } else {
                        current_digest.clone().unwrap()
                    };

                    // Reconstruct the full list of nodes at this level
                    let mut nodes = Vec::with_capacity(siblings.len() + 1);

                    // Insert siblings and current digest in correct positions
                    let mut sibling_idx = 0;
                    for i in 0..=siblings.len() {
                        if i == *position {
                            nodes.push(current.clone());
                        } else if sibling_idx < siblings.len() {
                            nodes.push(siblings[sibling_idx].clone());
                            sibling_idx += 1;
                        }
                    }

                    // Compute the combined digest
                    current_digest = Some(D::digest_items(&nodes));
                    current_is_raw = false;
                }
                PathElement::RawSiblings { position, siblings } => {
                    if level_idx == 0 {
                        // First level: siblings are raw items
                        let mut raw_items: Vec<&[u8]> = Vec::with_capacity(siblings.len() + 1);

                        // Insert item and siblings in correct positions
                        let mut sibling_idx = 0;
                        for i in 0..=siblings.len() {
                            if i == *position {
                                raw_items.push(item.as_ref());
                            } else if sibling_idx < siblings.len() {
                                raw_items.push(&siblings[sibling_idx]);
                                sibling_idx += 1;
                            }
                        }

                        // Compute digest from raw items
                        current_digest = Some(D::digest_items(&raw_items));
                        current_is_raw = false;
                    } else {
                        // Higher levels: siblings are already digests stored as raw bytes
                        // Get current value as digest
                        let current = if current_is_raw {
                            D::digest_item(&current_raw.as_ref().unwrap())
                        } else {
                            current_digest.clone().unwrap()
                        };

                        // Treat raw bytes as digest values and combine them
                        let mut all_items: Vec<&[u8]> = Vec::with_capacity(siblings.len() + 1);

                        // Insert current and siblings in correct positions
                        let mut sibling_idx = 0;
                        for i in 0..=siblings.len() {
                            if i == *position {
                                all_items.push(current.as_ref());
                            } else if sibling_idx < siblings.len() {
                                all_items.push(&siblings[sibling_idx]);
                                sibling_idx += 1;
                            }
                        }

                        // Compute combined digest
                        current_digest = Some(D::digest_items(&all_items));
                        current_is_raw = false;
                    }
                }
            }
        }

        // Get final digest
        let final_digest = if current_is_raw {
            D::digest_item(&current_raw.as_ref().unwrap())
        } else {
            current_digest.unwrap()
        };

        &final_digest == expected_root
    }
}


impl<T: Clone + AsRef<[u8]>, D: Digest> MembershipProof<T, D> {
    /// Verify the proof
    pub fn verify(&self) -> bool {
        self.path.verify(&self.item, &self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::mock::MockDigest;

    #[test]
    fn test_empty_proof_path() {
        let path: ProofPath<MockDigest> = ProofPath::new();
        assert_eq!(path.elements.len(), 0);
    }

    #[test]
    fn test_proof_path_construction() {
        let mut path: ProofPath<MockDigest> = ProofPath::new();
        path.add_siblings(0, vec![vec![1, 2, 3], vec![4, 5, 6]]);
        path.add_siblings(1, vec![vec![7, 8, 9]]);
        assert_eq!(path.elements.len(), 2);
    }

    #[test]
    fn test_proof_verification_single_level() {
        let path: ProofPath<MockDigest> = ProofPath::new();
        let item = b"A";
        let expected_root = MockDigest::digest_item(&item);

        assert!(path.verify(&item, &expected_root));
    }

    #[test]
    fn test_proof_verification_with_siblings() {
        let mut path: ProofPath<MockDigest> = ProofPath::new();
        let item = b"A";

        // A is at position 0, with siblings B and C
        // Store sibling digests, not raw values
        let b_digest = MockDigest::digest_item(&b"B");
        let c_digest = MockDigest::digest_item(&b"C");
        path.add_siblings(0, vec![b_digest, c_digest]);

        // Expected root is digest_items([A, B, C])
        // But since we store digests in the path, we need to compute accordingly
        let expected_root = MockDigest::digest_items(&[
            &MockDigest::digest_item(&item),
            &MockDigest::digest_item(&b"B"),
            &MockDigest::digest_item(&b"C"),
        ]);

        assert!(path.verify(&item, &expected_root));
    }
}
