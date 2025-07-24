# LazyTower-rs

A Rust implementation of LazyTower - an O(1) amortized cost alternative to Incremental Merkle Trees, designed for efficient membership proofs.

## Overview

LazyTower is a data structure that provides:
- **O(1) amortized append operations**
- **Efficient membership proof generation**
- **Configurable width (branching factor)**
- **Support for different hash functions**

## Features

- Generic over item types and digest functions
- Configurable tower width (default: 4)
- Test-driven development with comprehensive test coverage
- Support for SHA256 (optional feature)
- Clear separation between items and digests using enum types

## Usage

```rust
use lazytower_rs::{LazyTower, Digest};

// Create a new LazyTower with width 4
let mut tower: LazyTower<Vec<u8>, YourDigest> = LazyTower::new(4)?;

// Append items
tower.append(b"item1".to_vec());
tower.append(b"item2".to_vec());

// Get the root digest
let root = tower.root_digest();

// Generate membership proof (when implemented)
match tower.generate_proof(0) {
    Ok(proof) => {
        println!("Generated proof for item at index 0");
    }
    Err(e) => {
        println!("Proof generation error: {:?}", e);
    }
}
```

## Architecture

The LazyTower uses a level-based structure where:
- Items are appended to level 0
- When a level reaches the configured width, it overflows
- Overflow creates a digest that's added to the next level
- This process continues recursively

### Key Components

1. **`TowerNode<T, D>`**: Enum that can hold either an item or a digest
2. **`LazyTower<T, D>`**: Main structure with levels and item tracking
3. **`Digest` trait**: Abstraction for hash functions
4. **Proof generation**: Support for membership proofs (partial implementation)

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
- ✅ Digest computation with overflow handling
- ✅ Root digest computation
- ✅ Basic proof verification
- ✅ Error handling with Result types
- ✅ Item storage and position tracking
- ⚠️  Proof generation (basic implementation, limited by digest computation issues)
- ⚠️  Proof verification (works for simple cases only)

## Known Limitations

1. **Digest Computation**: The current implementation uses `digest_items()` for level overflow, which produces different results than combining individual digests. This prevents proper Merkle proof verification in overflow cases.

2. **Proof Generation**: Only supports basic cases. Complex multi-level proofs after multiple overflows are not fully implemented.

3. **Data Preservation**: While items are stored for proof generation, the relationship between items and their digests after overflow could be better tracked.

See `docs/improved_design.md` for proposed solutions to these limitations.

## Future Improvements

1. Fix digest computation to use pairwise combination for proper Merkle proofs
2. Complete proof generation with full overflow tracking
3. Implement Poseidon hash for ZK applications
4. Add serialization support
5. Optimize memory usage for large towers
6. Add benchmarks for performance analysis