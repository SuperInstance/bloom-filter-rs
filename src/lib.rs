//! # bloom-filter-rs
//!
//! Bloom filter implementations in pure Rust with zero external dependencies.
//!
//! ## Filters
//!
//! - **Classic** — Standard Bloom filter with configurable hash functions.
//! - **Counting** — Bloom filter with counters (supports deletion).
//! - **Scalable** — Grows automatically to maintain target false-positive rate.
//! - **Optimal** — Utilities for computing optimal `k` and `m` parameters.
//!
//! ## Example
//!
//! ```
//! use bloom_filter_rs::classic::BloomFilter;
//!
//! let mut bf = BloomFilter::new(1000, 0.01);
//! bf.insert(b"hello");
//! bf.insert(b"world");
//! assert!(bf.contains(b"hello"));
//! assert!(bf.contains(b"world"));
//! assert!(!bf.contains(b"missing"));
//! ```

pub mod classic;
pub mod counting;
pub mod scalable;
pub mod optimal;
pub mod hash;

pub use classic::BloomFilter;
pub use counting::CountingBloomFilter;
pub use scalable::ScalableBloomFilter;
