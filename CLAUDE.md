# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build
```bash
cargo build
cargo build --features sha256  # Build with SHA256 support
```

### Test
```bash
cargo test                     # Run all tests
cargo test --features sha256   # Run tests with SHA256 feature
cargo test -- --nocapture      # Run tests with output
cargo test <test_name>         # Run specific test
```

### Format & Lint
```bash
cargo fmt                      # Format code
cargo fmt --check             # Check formatting
cargo clippy                  # Run linter
cargo clippy -- -D warnings   # Treat warnings as errors
```

## Architecture

### Core Components

1. **LazyTower** (`src/tower.rs`): Main data structure with O(1) amortized append
   - Uses level-based structure where items overflow to higher levels
   - Each level has a configurable width (default: 4)
   - When a level reaches width, it creates a digest and adds to next level

2. **TowerNode Enum**: Represents either an Item or Digest in the tower
   - Allows mixing raw items and computed digests at different levels
   - Key to the lazy evaluation strategy

3. **Digest Trait** (`src/digest.rs`): Abstraction for hash functions
   - `digest_item()`: Hash single item
   - `digest_items()`: Hash multiple items (for level overflow)
   - `combine()`: Combine two digests (for Merkle tree construction)
   - Includes MockDigest for testing and optional SHA256 implementation

4. **Proof System** (`src/proof.rs`): Membership proof generation and verification
   - ProofPath with Left/Right siblings for Merkle path
   - Note: Proof generation is partially implemented (needs position tracking)

### Test Structure

Tests are organized by functionality:
- `tests/append_tests.rs`: Item appending and overflow behavior
- `tests/digest_tests.rs`: Digest computation and overflow handling
- `tests/root_digest_tests.rs`: Root digest calculation
- `tests/proof_tests.rs`: Proof generation and verification

### Key Design Decisions

- Generic over item types (T) and digest functions (D)
- Items must implement `AsRef<[u8]>` for hashing
- Levels are stored bottom-up (level 0 is where new items are appended)
- Root digest computation walks levels from bottom to top, combining nodes