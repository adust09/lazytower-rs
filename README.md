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
use lazytower_rs::{LazyTower, Digest};

// Create a new LazyTower with width 4
let mut tower: LazyTower<Vec<u8>, YourDigest> = LazyTower::new(4);

// Append items
tower.append(b"item1".to_vec());
tower.append(b"item2".to_vec());

// Get the root digest
let root = tower.root_digest();
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
