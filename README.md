## Overview

[LazyTower](https://ethresear.ch/t/lazytower-an-o-1-replacement-for-incremental-merkle-trees/21683) is a Incremental Merkle Trees that provides:
- **O(1) amortized append operations**
- **Efficient proof generation**
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

// Generate and verify proof
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
