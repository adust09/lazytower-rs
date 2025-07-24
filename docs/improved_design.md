# Improved LazyTower Design for Proper Proof Generation

## Current Issues

1. **Digest Computation Inconsistency**: 
   - `digest_items([A,B])` produces different output than `combine(digest(A), digest(B))`
   - This prevents proper Merkle proof verification

2. **Data Loss During Overflow**:
   - Original items are cleared when a level overflows
   - No way to reconstruct the exact computation path

3. **Limited Proof Generation**:
   - Only works for simple cases
   - Cannot handle multi-level proofs after multiple overflows

## Proposed Solution

### 1. Consistent Digest Computation

Instead of using `digest_items` for overflow, we should:
- Compute individual digests first: `d_a = digest(A)`, `d_b = digest(B)`
- Then combine them pairwise: `combine(d_a, d_b)`
- This ensures proof verification works correctly

### 2. Comprehensive Position Tracking

Track not just current position but the entire path:
```rust
struct ItemPath {
    original_index: usize,
    // Path from item to current position
    // Each step records: (level, index, siblings)
    path: Vec<PathStep>,
}

struct PathStep {
    level: usize,
    position: usize,
    siblings: Vec<SiblingInfo>,
}
```

### 3. Overflow History

Maintain complete overflow history:
```rust
struct OverflowEvent {
    timestamp: usize,  // When it happened (item count)
    level: usize,
    items: Vec<ItemOrDigest>,
    resulting_digest: D::Output,
}
```

### 4. Proof Generation Algorithm

1. Find the item's original position
2. Trace through overflow events to current position
3. Collect all necessary siblings along the path
4. Build proof path from bottom to top

### Implementation Steps

1. Refactor digest computation to use pairwise combination
2. Implement comprehensive position tracking
3. Add overflow event history
4. Rewrite proof generation to use the tracking data
5. Add tests for complex multi-level scenarios

## Benefits

- Correct Merkle proof verification
- Support for proofs at any point in time
- No data loss - can reconstruct entire history
- Extensible for future features (e.g., proof of non-membership)