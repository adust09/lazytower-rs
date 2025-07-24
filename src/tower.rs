//! Core LazyTower implementation

use crate::digest::Digest;
use crate::proof::MembershipProof;
use std::marker::PhantomData;

/// A node in the tower that can be either an item or a digest
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TowerNode<T, D: Digest> {
    /// A regular item
    Item(T),
    /// A digest of items from a lower level
    Digest(D::Output),
}

impl<T: AsRef<[u8]>, D: Digest> AsRef<[u8]> for TowerNode<T, D> {
    fn as_ref(&self) -> &[u8] {
        match self {
            TowerNode::Item(item) => item.as_ref(),
            TowerNode::Digest(digest) => digest.as_ref(),
        }
    }
}

/// LazyTower data structure with configurable width
#[derive(Debug, Clone)]
pub struct LazyTower<T, D: Digest> {
    /// Width of the tower (number of items per level before overflow)
    width: usize,
    /// Levels of the tower, where levels[0] is the bottom level
    levels: Vec<Vec<TowerNode<T, D>>>,
    /// Total number of items appended
    item_count: usize,
    /// Phantom data for digest type
    _digest: PhantomData<D>,
}

impl<T: Clone + AsRef<[u8]>, D: Digest> LazyTower<T, D> {
    /// Create a new empty LazyTower with the specified width
    pub fn new(width: usize) -> Self {
        assert!(width > 1, "Tower width must be greater than 1");
        Self { width, levels: vec![Vec::new()], item_count: 0, _digest: PhantomData }
    }

    /// Create a new LazyTower with default width of 4
    pub fn with_default_width() -> Self {
        Self::new(4)
    }

    /// Get the current height of the tower (number of levels)
    pub fn height(&self) -> usize {
        self.levels.len()
    }

    /// Get the total number of items in the tower
    pub fn len(&self) -> usize {
        self.item_count
    }

    /// Check if the tower is empty
    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Get the width of the tower
    pub fn width(&self) -> usize {
        self.width
    }

    /// Append an item to the tower (O(1) amortized)
    pub fn append(&mut self, item: T) {
        self.item_count += 1;
        self.append_to_level(0, TowerNode::Item(item));
    }

    /// Recursive helper to append a node to a specific level
    fn append_to_level(&mut self, level: usize, node: TowerNode<T, D>) {
        // Ensure we have enough levels
        while self.levels.len() <= level {
            self.levels.push(Vec::new());
        }

        // Add the node to the current level
        self.levels[level].push(node);

        // Check if the level overflows
        if self.levels[level].len() >= self.width {
            // Compute digest of the full level
            let digest = D::digest_items(&self.levels[level]);

            // Clear the current level
            self.levels[level].clear();

            // Recursively add the digest to the next level
            self.append_to_level(level + 1, TowerNode::Digest(digest));
        }
    }

    /// Get a reference to a specific level
    pub fn level(&self, index: usize) -> Option<&Vec<TowerNode<T, D>>> {
        self.levels.get(index)
    }

    /// Get all levels (for testing and debugging)
    #[cfg(test)]
    pub fn levels(&self) -> &Vec<Vec<TowerNode<T, D>>> {
        &self.levels
    }

    /// Compute the root digest of the tower
    pub fn root_digest(&self) -> Option<D::Output> {
        // Find the highest non-empty level
        for level in self.levels.iter().rev() {
            if !level.is_empty() {
                // If there's only one node at this level, return its digest
                if level.len() == 1 {
                    return Some(match &level[0] {
                        TowerNode::Item(item) => D::digest_item(item),
                        TowerNode::Digest(digest) => digest.clone(),
                    });
                } else {
                    // Multiple nodes at the top level - compute their combined digest
                    return Some(D::digest_items(level));
                }
            }
        }
        None
    }

    /// Store the original items for proof generation
    /// Note: In a production implementation, this would be stored more efficiently
    #[cfg(test)]
    pub fn store_item(&mut self, _index: usize, _item: T) {
        // For testing purposes, we'll implement a simple storage mechanism
        // In production, this would be handled differently
    }

    /// Generate a membership proof for an item at a given index
    /// Note: This is a simplified implementation for testing
    /// In production, you'd need to track item positions through overflows
    pub fn generate_proof(&self, _index: usize) -> Option<MembershipProof<T, D>> {
        // TODO: Implement actual proof generation
        // This requires tracking item positions through the tower structure
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::mock::MockDigest;

    #[test]
    fn test_new_tower() {
        let tower: LazyTower<Vec<u8>, MockDigest> = LazyTower::new(4);
        assert_eq!(tower.width(), 4);
        assert_eq!(tower.height(), 1);
        assert_eq!(tower.len(), 0);
        assert!(tower.is_empty());
    }

    #[test]
    #[should_panic(expected = "Tower width must be greater than 1")]
    fn test_invalid_width() {
        let _tower: LazyTower<Vec<u8>, MockDigest> = LazyTower::new(1);
    }
}
