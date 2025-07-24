# LazyTower-rs

A Rust implementation of [LazyTower](https://ethresear.ch/t/lazytower-an-o-1-replacement-for-incremental-merkle-trees/21683) - an O(1) amortized cost alternative to Incremental Merkle Trees, designed for efficient membership proofs.

## Overview
LazyTower is a data structure that provides:
- **O(1) amortized append operations**
- **Efficient membership proof generation**
- **Configurable width (branching factor)**
- **Support for different hash functions**

The LazyTower uses a level-based structure where:
- Items are appended to level 0
- When a level reaches the configured width, it overflows
- Overflow creates a digest that's added to the next level
- This process continues recursively

## Usage

```rust
use lazytower_rs::{LazyTower, MockDigest};

// Create a new LazyTower with width 4
let mut tower: LazyTower<Vec<u8>, MockDigest> = LazyTower::new(4)?;

// Append items
tower.append(b"item1".to_vec());
tower.append(b"item2".to_vec());

// Get the root digest
let root = tower.root_digest();

// Generate and verify membership proof
match tower.generate_proof(0) {
    Ok(proof) => {
        println!("Generated proof for item at index 0");
        
        // Verify the proof
        if proof.verify(&b"item1".to_vec(), &root, &MockDigest) {
            println!("Proof verified successfully!");
        }
    }
    Err(e) => {
        println!("Proof generation error: {:?}", e);
    }
}
```

## Testing

Run all tests:
```bash
cargo test
```

Run tests with SHA256 feature:
```bash
cargo test --features sha256
```

## Implementation Status

- ✅ Core data structures
- ✅ O(1) amortized append operation
- ✅ Pairwise digest computation for proper Merkle trees
- ✅ Root digest computation
- ✅ Membership proof generation
- ✅ Proof verification
- ✅ Error handling with Result types
- ✅ Item storage and position tracking
- ✅ Overflow tracking for proof generation

**Test Coverage**: 54 out of 55 tests passing (98.2%)

## Known Limitations

1. **Edge Case in Deep Structures**: There's one known edge case when generating proofs for deeply nested structures (8+ items with width 2) where the sibling ordering logic may produce incorrect results. This affects 1 out of 55 tests.

2. **Heuristic-Based Sibling Determination**: The current implementation uses position-based heuristics for determining siblings at higher levels, which may not work for all complex tree structures.

## Architecture Highlights

- **Generic Design**: Works with any item type `T` that implements `AsRef<[u8]>` and any digest function `D` implementing the `Digest` trait
- **Lazy Evaluation**: Digests are only computed when levels overflow or when the root digest is requested
- **Merkle Tree Construction**: Uses pairwise combination during overflow to build balanced Merkle trees
- **Proof System**: Complete proof generation and verification with path reconstruction from stored items

## Future Improvements

1. Resolve the edge case in deep nested structures
2. Implement more robust sibling determination logic
3. Add Poseidon hash for ZK applications
4. Add serialization support
5. Optimize memory usage for large towers
6. Add benchmarks for performance analysis

