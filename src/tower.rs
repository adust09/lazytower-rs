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

/// Node identifier for tracking nodes through levels
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum NodeId {
    /// Original item by index
    Item(usize),
    /// Digest created from other nodes
    Digest(Vec<NodeId>),
}

/// Overflow record to track which items were digested together
#[derive(Debug, Clone)]
struct OverflowRecord<D: Digest> {
    /// The level that overflowed
    level: usize,
    /// The node IDs that were digested together
    node_ids: Vec<NodeId>,
    /// The resulting digest
    result_digest: D::Output,
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
    overflow_records: Vec<OverflowRecord<D>>,
    /// Mapping from digest to the NodeIds it contains
    digest_to_nodes: HashMap<Vec<u8>, Vec<NodeId>>,
    /// Mapping from level and index to NodeId for current nodes
    level_nodes: HashMap<(usize, usize), NodeId>,
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
            digest_to_nodes: HashMap::new(),
            level_nodes: HashMap::new(),
            _digest: PhantomData,
        })
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
        let position = ItemPosition {
            level: 0,
            index: self.levels[0].len(),
        };
        self.item_positions.insert(item_index, position.clone());

        // Track the node ID
        let node_id = NodeId::Item(item_index);
        self.level_nodes
            .insert((position.level, position.index), node_id.clone());

        self.append_to_level(0, TowerNode::Item(item), node_id);
    }

    /// Recursive helper to append a node to a specific level
    fn append_to_level(&mut self, level: usize, node: TowerNode<T, D>, node_id: NodeId) {
        // Ensure we have enough levels
        while self.levels.len() <= level {
            self.levels.push(Vec::new());
        }

        // Add the node to the current level
        let node_index = self.levels[level].len();
        self.levels[level].push(node);

        // Track node at this position
        self.level_nodes
            .insert((level, node_index), node_id.clone());

        // Check if the level overflows
        if self.levels[level].len() >= self.width {
            // Collect node IDs that will be digested
            let mut overflow_node_ids = Vec::new();
            for i in 0..self.width {
                if let Some(nid) = self.level_nodes.get(&(level, i)) {
                    overflow_node_ids.push(nid.clone());
                }
            }

            // Compute digest of the full level
            let digest = D::digest_items(&self.levels[level]);
            let digest_bytes = digest.as_ref().to_vec();

            // Create new node ID for the digest
            let digest_node_id = NodeId::Digest(overflow_node_ids.clone());

            // Track which nodes went into this digest
            self.digest_to_nodes
                .insert(digest_bytes.clone(), overflow_node_ids.clone());

            // Track overflow record
            self.overflow_records.push(OverflowRecord {
                level,
                node_ids: overflow_node_ids,
                result_digest: digest.clone(),
            });

            // Update positions for items at level 0
            if level == 0 {
                // Extract item indices from overflow node IDs
                for node_id in &self.overflow_records.last().unwrap().node_ids {
                    if let NodeId::Item(idx) = node_id {
                        if let Some(pos) = self.item_positions.get_mut(idx) {
                            pos.level = level + 1;
                            pos.index = self.levels.get(level + 1).map_or(0, |l| l.len());
                        }
                    }
                }
            }

            // Clear the current level and its node mappings
            self.levels[level].clear();
            for i in 0..self.width {
                self.level_nodes.remove(&(level, i));
            }

            // Recursively add the digest to the next level
            self.append_to_level(level + 1, TowerNode::Digest(digest), digest_node_id);
        }
    }

    /// Get a reference to a specific level
    pub fn level(&self, index: usize) -> Option<&Vec<TowerNode<T, D>>> {
        self.levels.get(index)
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

    /// Generate a proof for an item at a given index
    pub fn generate_proof(&self, index: usize) -> Result<MembershipProof<T, D>, LazyTowerError> {
        // Check bounds
        if self.item_count == 0 || index >= self.item_count {
            return Err(LazyTowerError::InvalidIndex {
                index,
                max: self.item_count,
            });
        }

        // Get the original item
        let item = self
            .items
            .get(&index)
            .ok_or(LazyTowerError::ProofGenerationNotImplemented)?
            .clone();

        // Get current root
        let root = self
            .root_digest()
            .ok_or(LazyTowerError::ProofGenerationNotImplemented)?;

        // Build the proof path
        let mut path = ProofPath::new();

        // Simple case: if there's only one item, no siblings needed
        if self.item_count == 1 {
            return Ok(MembershipProof { item, path, root });
        }

        // Build proof path from item to root using NodeId tracking
        let item_node_id = NodeId::Item(index);
        self.build_proof_path_recursive(&item_node_id, &mut path)?;

        Ok(MembershipProof { item, path, root })
    }

    /// Recursively build proof path for a node
    fn build_proof_path_recursive(
        &self,
        node_id: &NodeId,
        path: &mut ProofPath<D>,
    ) -> Result<(), LazyTowerError> {
        // Find which overflow record contains this node
        for record in &self.overflow_records {
            if record.node_ids.contains(node_id) {
                // Find position and siblings within this overflow group
                let mut position = 0;

                if record.level == 0 {
                    // Level 0: Use raw siblings (actual item values)
                    let mut raw_siblings = Vec::new();

                    for (i, nid) in record.node_ids.iter().enumerate() {
                        if nid == node_id {
                            position = i;
                        } else if let NodeId::Item(idx) = nid {
                            if let Some(item) = self.items.get(idx) {
                                raw_siblings.push(item.as_ref().to_vec());
                            }
                        }
                    }

                    path.add_raw_siblings(position, raw_siblings);
                } else {
                    // Higher levels: Use digest siblings
                    let mut digest_siblings = Vec::new();

                    for (i, nid) in record.node_ids.iter().enumerate() {
                        if nid == node_id {
                            position = i;
                        } else if let NodeId::Digest(child_nodes) = nid {
                            // Find the digest for this node group
                            for overflow_record in &self.overflow_records {
                                if &overflow_record.node_ids == child_nodes {
                                    digest_siblings.push(overflow_record.result_digest.clone());
                                    break;
                                }
                            }
                        }
                    }

                    path.add_siblings(position, digest_siblings);
                }

                // Continue building path for the parent digest
                let parent_node_id = NodeId::Digest(record.node_ids.clone());
                return self.build_proof_path_recursive(&parent_node_id, path);
            }
        }

        // If not in any overflow record, check if it's currently at a level
        for ((level, index), nid) in &self.level_nodes {
            if nid == node_id {
                // Found the node at a current level
                if let Some(level_nodes) = self.levels.get(*level) {
                    if level_nodes.len() > 1 {
                        // Has siblings at this level
                        let mut siblings = Vec::new();
                        for (i, node) in level_nodes.iter().enumerate() {
                            if i != *index {
                                let raw_bytes = match node {
                                    TowerNode::Item(item) => item.as_ref().to_vec(),
                                    TowerNode::Digest(d) => d.as_ref().to_vec(),
                                };
                                siblings.push(raw_bytes);
                            }
                        }
                        path.add_raw_siblings(*index, siblings);
                    }
                }
                return Ok(());
            }
        }

        Ok(())
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
