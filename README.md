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
let mut tower: LazyTower<Vec<u8>, YourDigest> = LazyTower::new(4);

// Append items
tower.append(b"item1".to_vec());
tower.append(b"item2".to_vec());

// Get the root digest
let root = tower.root_digest();
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
- ⚠️  Proof generation (partial - needs position tracking)

## Future Improvements

1. Complete proof generation with item position tracking
2. Implement Poseidon hash for ZK applications
3. Add serialization support
4. Optimize memory usage for large towers
5. Add benchmarks for performance analysis