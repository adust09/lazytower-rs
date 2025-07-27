# Multi-Level Proof Generation Limitation

## Current Status

The current implementation of LazyTower has a limitation in proof generation for items that have participated in multiple levels of overflow.

### What Works

- Proof generation for items at level 0 (before any overflow)
- Proof generation for items that participated in a single level 0 overflow
- Basic tower operations (append, overflow, root digest computation)

### What Doesn't Work

- Proof generation for items that have overflowed multiple times (e.g., from level 0 to level 1, then level 1 to level 2)
- Proof verification for items in complex multi-level structures

### Example

```rust
let mut tower: LazyTower<Vec<u8>, MockDigest> = LazyTower::new(2).unwrap();

// Add 4 items - creates multi-level structure
for i in 0..4 {
    tower.append(vec![i]);
}

// Tower structure:
// - Items 0,1 overflow to level 1 as digest
// - Items 2,3 overflow to level 1 as digest  
// - Level 1 overflows to level 2

// Currently, proofs for items 0-3 cannot be verified against the root at level 2
```

## Technical Details

The issue is in `src/tower.rs` at line 263 where there's a TODO comment:
```rust
// TODO: Handle multiple levels of overflow
```

To properly implement multi-level proof generation, the following would be needed:

1. Track overflow records for all levels (not just level 0)
2. Maintain a mapping from digests to the items they contain
3. Build proof paths that include all intermediate digests from the item to the root
4. Handle the case where items are at different levels in the tower

## Impact

This limitation means that LazyTower cannot currently be used as a full replacement for traditional Merkle trees in applications that require proofs for all items after multiple levels of aggregation.

For use cases where items are only aggregated once (single level overflow), the current implementation works correctly.