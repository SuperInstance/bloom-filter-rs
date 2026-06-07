//! Classic Bloom filter.
//!
//! A space-efficient probabilistic data structure for set membership testing.
//! False positives are possible; false negatives are not.

use crate::hash::hash_indices;
use crate::optimal::{optimal_k, optimal_m};

/// Classic Bloom filter.
pub struct BloomFilter {
    bits: Vec<u64>,
    num_bits: usize,
    num_hashes: usize,
    count: usize,
}

impl BloomFilter {
    /// Create a new Bloom filter sized for `expected_items` with a target
    /// false positive rate of `fp_rate`.
    pub fn new(expected_items: usize, fp_rate: f64) -> Self {
        let num_bits = optimal_m(expected_items, fp_rate).max(64);
        let num_hashes = optimal_k(num_bits, expected_items);
        let num_words = num_bits.div_ceil(64);
        Self {
            bits: vec![0u64; num_words],
            num_bits,
            num_hashes,
            count: 0,
        }
    }

    /// Create with explicit `num_bits` and `num_hashes`.
    pub fn with_params(num_bits: usize, num_hashes: usize) -> Self {
        let num_words = num_bits.div_ceil(64);
        Self {
            bits: vec![0u64; num_words],
            num_bits: num_bits.max(64),
            num_hashes: num_hashes.max(1),
            count: 0,
        }
    }

    /// Insert an item.
    pub fn insert(&mut self, item: &[u8]) {
        for idx in hash_indices(item, self.num_hashes, self.num_bits) {
            let word = idx / 64;
            let bit = idx % 64;
            self.bits[word] |= 1u64 << bit;
        }
        self.count += 1;
    }

    /// Check if an item is possibly in the set.
    ///
    /// Returns `true` if the item might be present (may be a false positive).
    /// Returns `false` if the item is definitely not present.
    pub fn contains(&self, item: &[u8]) -> bool {
        for idx in hash_indices(item, self.num_hashes, self.num_bits) {
            let word = idx / 64;
            let bit = idx % 64;
            if self.bits[word] & (1u64 << bit) == 0 {
                return false;
            }
        }
        true
    }

    /// Number of items inserted.
    pub fn count(&self) -> usize { self.count }

    /// Number of bits in the filter.
    pub fn num_bits(&self) -> usize { self.num_bits }

    /// Number of hash functions.
    pub fn num_hashes(&self) -> usize { self.num_hashes }

    /// Estimate current false positive rate based on fill level.
    pub fn estimated_fpr(&self) -> f64 {
        let k = self.num_hashes as f64;
        let n = self.count as f64;
        let m = self.num_bits as f64;
        (1.0 - (-k * n / m).exp()).powi(k as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_contains() {
        let mut bf = BloomFilter::new(100, 0.01);
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
        assert!(!bf.contains(b"world"));
    }

    #[test]
    fn no_false_negatives() {
        let mut bf = BloomFilter::new(100, 0.01);
        let items: Vec<&[u8]> = vec![b"alpha", b"beta", b"gamma", b"delta", b"epsilon"];
        for &item in &items {
            bf.insert(item);
        }
        for &item in &items {
            assert!(bf.contains(item), "false negative for {:?}", std::str::from_utf8(item).unwrap());
        }
    }

    #[test]
    fn count_tracking() {
        let mut bf = BloomFilter::new(100, 0.01);
        assert_eq!(bf.count(), 0);
        bf.insert(b"a");
        assert_eq!(bf.count(), 1);
        bf.insert(b"b");
        assert_eq!(bf.count(), 2);
    }

    #[test]
    fn false_positive_rate_matches_theory() {
        let n = 10_000;
        let target_fpr = 0.01;
        let mut bf = BloomFilter::new(n, target_fpr);
        // Insert n items
        for i in 0..n {
            let bytes = format!("item_{}", i);
            bf.insert(bytes.as_bytes());
        }
        // Test false positives on items NOT in the set
        let test_count = 100_000;
        let mut fp_count = 0;
        for i in n..(n + test_count) {
            let bytes = format!("item_{}", i);
            if bf.contains(bytes.as_bytes()) {
                fp_count += 1;
            }
        }
        let actual_fpr = fp_count as f64 / test_count as f64;
        // Should be within an order of magnitude of the target
        assert!(actual_fpr < target_fpr * 5.0, "actual_fpr={actual_fpr}, target={target_fpr}");
    }

    #[test]
    fn empty_filter_no_false_positives() {
        let bf = BloomFilter::new(100, 0.01);
        assert!(!bf.contains(b"anything"));
        assert!(!bf.contains(b"something"));
    }

    #[test]
    fn duplicate_insert_is_idempotent() {
        let mut bf = BloomFilter::new(100, 0.01);
        bf.insert(b"hello");
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
        assert_eq!(bf.count(), 2); // count increases but filter is same
    }

    #[test]
    fn with_params() {
        let bf = BloomFilter::with_params(1024, 5);
        assert_eq!(bf.num_bits(), 1024);
        assert_eq!(bf.num_hashes(), 5);
    }

    #[test]
    fn estimated_fpr_increases_with_insertions() {
        let mut bf = BloomFilter::new(100, 0.01);
        let fpr_0 = bf.estimated_fpr();
        for i in 0..100 {
            bf.insert(format!("x{}", i).as_bytes());
        }
        let fpr_100 = bf.estimated_fpr();
        assert!(fpr_100 > fpr_0, "fpr_100={fpr_100}, fpr_0={fpr_0}");
    }

    #[test]
    fn large_filter() {
        let mut bf = BloomFilter::new(1_000_000, 0.001);
        for i in 0..1000 {
            bf.insert(format!("big_{}", i).as_bytes());
        }
        assert!(bf.contains(b"big_0"));
        assert!(bf.contains(b"big_999"));
        assert!(!bf.contains(b"big_1000"));
    }

    #[test]
    fn string_items() {
        let mut bf = BloomFilter::new(100, 0.01);
        bf.insert("hello world".as_bytes());
        assert!(bf.contains("hello world".as_bytes()));
        assert!(!bf.contains("goodbye world".as_bytes()));
    }

    #[test]
    fn num_bits_and_hashes() {
        let bf = BloomFilter::new(1000, 0.01);
        assert!(bf.num_bits() > 0);
        assert!(bf.num_hashes() > 0);
    }
}
