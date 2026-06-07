//! Scalable Bloom filter.
//!
//! Automatically grows by adding new Bloom filter partitions when the current
//! one exceeds its capacity. Maintains a target false positive rate across
//! all partitions.

use crate::classic::BloomFilter;


/// Scalable Bloom filter that grows as needed.
pub struct ScalableBloomFilter {
    filters: Vec<BloomFilter>,
    target_fpr: f64,
    growth_factor: usize,
    items_per_partition: usize,
    current_count: usize,
}

impl ScalableBloomFilter {
    /// Create a new scalable Bloom filter.
    ///
    /// * `initial_items` — Expected items per partition.
    /// * `fp_rate`       — Target false positive rate.
    /// * `growth_factor` — Each new partition is this many × larger.
    pub fn new(initial_items: usize, fp_rate: f64, growth_factor: usize) -> Self {
        let mut s = Self {
            filters: Vec::new(),
            target_fpr: fp_rate,
            growth_factor,
            items_per_partition: initial_items,
            current_count: 0,
        };
        s.add_partition(initial_items, fp_rate);
        s
    }

    fn add_partition(&mut self, items: usize, fpr: f64) {
        self.filters.push(BloomFilter::new(items, fpr));
    }

    /// Insert an item.
    pub fn insert(&mut self, item: &[u8]) {
        let last = self.filters.last().unwrap();
        if last.count() >= self.items_per_partition {
            // Add a new, larger partition with tighter FPR
            let new_items = self.items_per_partition * self.growth_factor;
            let new_fpr = self.target_fpr / 2.0;
            self.add_partition(new_items, new_fpr);
            self.items_per_partition = new_items;
        }
        self.filters.last_mut().unwrap().insert(item);
        self.current_count += 1;
    }

    /// Check if an item is possibly in the set (checks all partitions).
    pub fn contains(&self, item: &[u8]) -> bool {
        self.filters.iter().any(|f| f.contains(item))
    }

    /// Total number of items inserted.
    pub fn count(&self) -> usize { self.current_count }

    /// Number of partitions.
    pub fn num_partitions(&self) -> usize { self.filters.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_insert_contains() {
        let mut bf = ScalableBloomFilter::new(100, 0.01, 2);
        bf.insert(b"hello");
        assert!(bf.contains(b"hello"));
        assert!(!bf.contains(b"world"));
    }

    #[test]
    fn scales_with_many_inserts() {
        let mut bf = ScalableBloomFilter::new(50, 0.01, 2);
        for i in 0..500 {
            bf.insert(format!("item_{}", i).as_bytes());
        }
        assert!(bf.num_partitions() > 1, "should have scaled");
        // Check no false negatives
        for i in 0..500 {
            assert!(bf.contains(format!("item_{}", i).as_bytes()), "false negative for {i}");
        }
    }

    #[test]
    fn count_tracking() {
        let mut bf = ScalableBloomFilter::new(100, 0.01, 2);
        assert_eq!(bf.count(), 0);
        for i in 0..10 {
            bf.insert(format!("x{}", i).as_bytes());
        }
        assert_eq!(bf.count(), 10);
    }

    #[test]
    fn false_positive_rate_stays_bounded() {
        let mut bf = ScalableBloomFilter::new(100, 0.01, 2);
        for i in 0..500 {
            bf.insert(format!("item_{}", i).as_bytes());
        }
        let test_count = 10_000;
        let mut fp = 0;
        for i in 500..(500 + test_count) {
            if bf.contains(format!("item_{}", i).as_bytes()) {
                fp += 1;
            }
        }
        let actual_fpr = fp as f64 / test_count as f64;
        assert!(actual_fpr < 0.1, "actual_fpr={actual_fpr}");
    }

    #[test]
    fn single_partition_for_few_items() {
        let mut bf = ScalableBloomFilter::new(1000, 0.01, 2);
        for i in 0..50 {
            bf.insert(format!("x{}", i).as_bytes());
        }
        assert_eq!(bf.num_partitions(), 1);
    }

    #[test]
    fn partition_growth() {
        let mut bf = ScalableBloomFilter::new(10, 0.01, 2);
        for i in 0..100 {
            bf.insert(format!("x{}", i).as_bytes());
        }
        assert!(bf.num_partitions() >= 3);
    }

    #[test]
    fn empty_contains_nothing() {
        let bf = ScalableBloomFilter::new(100, 0.01, 2);
        // The initial partition is empty
        assert!(!bf.contains(b"anything"));
        assert_eq!(bf.count(), 0);
    }
}
