//! Membership proof structures and generation

use crate::digest::Digest;

/// A path element in a membership proof
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathElement<D: Digest> {
    /// Left sibling with its digest
    Left(D::Output),
    /// Right sibling with its digest
    Right(D::Output),
}

/// A membership proof path from item to root
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofPath<D: Digest> {
    /// The path elements from bottom to top
    pub elements: Vec<PathElement<D>>,
}

/// A complete membership proof
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

    /// Add a left sibling to the path
    pub fn add_left(&mut self, digest: D::Output) {
        self.elements.push(PathElement::Left(digest));
    }

    /// Add a right sibling to the path
    pub fn add_right(&mut self, digest: D::Output) {
        self.elements.push(PathElement::Right(digest));
    }

    /// Verify a proof path for an item
    pub fn verify<T: AsRef<[u8]>>(&self, item: &T, expected_root: &D::Output) -> bool {
        let mut current = D::digest_item(item);

        for element in &self.elements {
            current = match element {
                PathElement::Left(left) => D::combine(left, &current),
                PathElement::Right(right) => D::combine(&current, right),
            };
        }

        &current == expected_root
    }
}

impl<D: Digest> Default for ProofPath<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + AsRef<[u8]>, D: Digest> MembershipProof<T, D> {
    /// Verify the membership proof
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
        path.add_left(vec![1, 2, 3]);
        path.add_right(vec![4, 5, 6]);
        assert_eq!(path.elements.len(), 2);
    }
}
