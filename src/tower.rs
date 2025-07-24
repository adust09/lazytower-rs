//! Core LazyTower implementation

use crate::digest::Digest;
use crate::error::LazyTowerError;
use crate::proof::{MembershipProof, ProofPath};
use std::collections::HashMap;
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

/// Position of an item in the tower
#[derive(Debug, Clone)]
struct ItemPosition {
    /// The level where the item or its digest resides
    level: usize,
    /// The index within that level
    index: usize,
}

/// Overflow record to track which items were digested together
#[derive(Debug, Clone)]
struct OverflowRecord {
    /// The level that overflowed
    #[allow(dead_code)]
    level: usize,
    /// The indices of items that were digested together
    item_indices: Vec<usize>,
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
    /// Storage for original items (for proof generation)
    items: HashMap<usize, T>,
    /// Mapping from item index to its current position in the tower
    item_positions: HashMap<usize, ItemPosition>,
    /// Overflow records to track digests
    overflow_records: Vec<OverflowRecord>,
    /// Phantom data for digest type
    _digest: PhantomData<D>,
}

impl<T: Clone + AsRef<[u8]>, D: Digest> LazyTower<T, D> {
    /// Create a new empty LazyTower with the specified width
    pub fn new(width: usize) -> Result<Self, LazyTowerError> {
        if width <= 1 {
            return Err(LazyTowerError::InvalidWidth { width });
        }
        Ok(Self { 
            width, 
            levels: vec![Vec::new()], 
            item_count: 0,
            items: HashMap::new(),
            item_positions: HashMap::new(),
            overflow_records: Vec::new(),
            _digest: PhantomData 
        })
    }

    /// Create a new LazyTower with default width of 4
    pub fn with_default_width() -> Result<Self, LazyTowerError> {
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
        let item_index = self.item_count;
        self.item_count += 1;
        
        // Store the item for proof generation
        self.items.insert(item_index, item.clone());
        
        // Track the initial position
        self.item_positions.insert(item_index, ItemPosition {
            level: 0,
            index: self.levels[0].len(),
        });
        
        self.append_to_level(0, TowerNode::Item(item), Some(item_index));
    }

    /// Recursive helper to append a node to a specific level
    fn append_to_level(&mut self, level: usize, node: TowerNode<T, D>, _initial_item_index: Option<usize>) {
        // Ensure we have enough levels
        while self.levels.len() <= level {
            self.levels.push(Vec::new());
        }

        // Add the node to the current level
        self.levels[level].push(node);

        // Check if the level overflows
        if self.levels[level].len() >= self.width {
            // Track overflow for proof generation
            if level == 0 {
                // For level 0, we track which original items are being digested
                let start_index = if self.item_count >= self.width {
                    self.item_count - self.width
                } else {
                    0
                };
                
                let mut overflow_items = Vec::new();
                for i in 0..self.width.min(self.item_count) {
                    let item_idx = start_index + i;
                    overflow_items.push(item_idx);
                    
                    // Update position to indicate it's now part of a digest at the next level
                    if let Some(pos) = self.item_positions.get_mut(&item_idx) {
                        pos.level = level + 1;
                        pos.index = self.levels.get(level + 1).map_or(0, |l| l.len());
                    }
                }
                
                self.overflow_records.push(OverflowRecord {
                    level,
                    item_indices: overflow_items,
                });
            }
            
            // Compute digest of the full level
            let digest = D::digest_items(&self.levels[level]);

            // Clear the current level
            self.levels[level].clear();

            // Recursively add the digest to the next level
            self.append_to_level(level + 1, TowerNode::Digest(digest), None);
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


    /// Generate a membership proof for an item at a given index
    pub fn generate_proof(&self, index: usize) -> Result<MembershipProof<T, D>, LazyTowerError> {
        // Check bounds
        if self.item_count == 0 || index >= self.item_count {
            return Err(LazyTowerError::InvalidIndex { index, max: self.item_count });
        }
        
        // Get the original item
        let item = self.items.get(&index)
            .ok_or(LazyTowerError::ProofGenerationNotImplemented)?
            .clone();
        
        // Get current root
        let root = self.root_digest()
            .ok_or(LazyTowerError::ProofGenerationNotImplemented)?;
        
        // Build the proof path
        let mut path = ProofPath::new();
        
        // Simple case: if there's only one item, no siblings needed
        if self.item_count == 1 {
            return Ok(MembershipProof { item, path, root });
        }
        
        // Check if the item has been part of an overflow
        let mut found_in_overflow = false;
        let mut sibling_indices = Vec::new();
        
        for record in &self.overflow_records {
            if record.item_indices.contains(&index) {
                found_in_overflow = true;
                // Find siblings in this overflow group
                for &idx in &record.item_indices {
                    if idx != index {
                        sibling_indices.push(idx);
                    }
                }
                break;
            }
        }
        
        if found_in_overflow && !sibling_indices.is_empty() {
            // Simple case: handle two items that overflowed together
            if sibling_indices.len() == 1 {
                let sibling_idx = sibling_indices[0];
                if let Some(sibling_item) = self.items.get(&sibling_idx) {
                    let sibling_digest = D::digest_item(sibling_item);
                    if index < sibling_idx {
                        path.add_right(sibling_digest);
                    } else {
                        path.add_left(sibling_digest);
                    }
                    
                    // TODO: Handle multiple levels of overflow
                    // For now, just return if we found the immediate sibling
                    return Ok(MembershipProof { item, path, root });
                }
            }
        }
        
        // Handle items still at level 0
        if let Some(pos) = self.item_positions.get(&index) {
            if pos.level == 0 {
                // Item is still at level 0, find siblings
                let level = &self.levels[0];
                
                // Simple case: two items at level 0
                if level.len() == 2 {
                    let sibling_idx = if pos.index == 0 { 1 } else { 0 };
                    if let Some(sibling) = level.get(sibling_idx) {
                        match sibling {
                            TowerNode::Item(item) => {
                                let digest = D::digest_item(item);
                                if pos.index == 0 {
                                    path.add_right(digest);
                                } else {
                                    path.add_left(digest);
                                }
                            }
                            TowerNode::Digest(d) => {
                                if pos.index == 0 {
                                    path.add_right(d.clone());
                                } else {
                                    path.add_left(d.clone());
                                }
                            }
                        }
                    }
                    return Ok(MembershipProof { item, path, root });
                }
            }
        }
        
        // For more complex cases, we need full implementation
        Err(LazyTowerError::ProofGenerationNotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::mock::MockDigest;

    #[test]
    fn test_new_tower() {
        let tower: LazyTower<Vec<u8>, MockDigest> = LazyTower::new(4).unwrap();
        assert_eq!(tower.width(), 4);
        assert_eq!(tower.height(), 1);
        assert_eq!(tower.len(), 0);
        assert!(tower.is_empty());
    }

    #[test]
    fn test_invalid_width() {
        let result: Result<LazyTower<Vec<u8>, MockDigest>, _> = LazyTower::new(1);
        assert!(result.is_err());
        match result {
            Err(LazyTowerError::InvalidWidth { width }) => assert_eq!(width, 1),
            _ => panic!("Expected InvalidWidth error"),
        }
    }
}
