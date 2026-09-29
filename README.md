# Rust Cohort Assignments

This repository contains the weekly assignments for a Rust cohort built around *Building Bitcoin in Rust* and the Rust Book.

The cohort uses 8 teaching weeks plus a 2-week capstone completion window. Weeks 1-2 quickly cover the Rust ground needed for the rest of the course, then Weeks 3-8 focus on Rust architectures that are useful for Bitcoin engineering.

Later weeks mirror the component boundaries from *Building Bitcoin in Rust* at a teaching level, but the assignment APIs, fixtures, and tests are original. Students should not copy code or CBOR/key files from the book repository.

Each week is a separate Cargo crate under `weeks/`. Starter code lives under that week's `src/` directory, and the visible tests live in that week's `tests/` directory.

## Weeks

- `weeks/week-01`: Rust syntax, primitives, vectors, hash maps, and simple Bitcoin data.
- `weeks/week-02`: Ownership, borrowing, string slices, basic lifetimes, and small utilities.
- `weeks/week-03`: Structs, enums, pattern matching, Serde derives, UUIDs, dispatch, and Bitcoin domain modeling.
- `weeks/week-04`: Extends the Week 3 model with parsing, validation, `thiserror`, SHA-256 hashing, hex decoding, and block-building helpers.
- `weeks/week-05`: Module boundaries, Serde JSON persistence, merkle roots, blockchain state, and utility binaries.
- `weeks/week-06`: Miner architecture with UTXO state, mempool selection, deterministic proof-of-work, and multi-threaded coordination.
- `weeks/week-07`: Node architecture with protocol parsing, peer state, request dispatch, bounded async channels, and TCP handling.
- `weeks/week-08`: Wallet architecture with UTXO selection, fee estimation, command parsing, node-client integration, async tasks, and structured logging.

## Running Tests

From the repository root:

```bash
cargo test --workspace
```

Run a single week:

```bash
cargo test -p week_01_foundations
cargo test -p week_02_ownership
cargo test -p week_03_modeling
cargo test -p week_04_errors_traits
cargo test -p week_05_persistence
cargo test -p week_06_miner
cargo test -p week_07_async_node
cargo test -p week_08_wallet_integration
```

Check formatting:

```bash
cargo fmt --all -- --check
```

Run clippy after Week 5 solutions:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

The starter code intentionally contains `todo!()` implementations. Tests are expected to fail until students complete the assignment.

