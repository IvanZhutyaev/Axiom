//! Bloom filter for SSTable key membership tests.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<u64>,
    k: usize,
    inserted: usize,
}

impl BloomFilter {
    pub fn with_capacity(expected: usize, fp_rate: f64) -> Self {
        let m = optimal_bits(expected, fp_rate).max(64);
        let k = optimal_k(m, expected).max(1);
        Self {
            bits: vec![0; (m + 63) / 64],
            k,
            inserted: 0,
        }
    }

    pub fn insert(&mut self, key: &[u8]) {
        for i in 0..self.k {
            let h = hash_key(key, i as u64);
            let bit = (h as usize) % (self.bits.len() * 64);
            self.bits[bit / 64] |= 1u64 << (bit % 64);
        }
        self.inserted += 1;
    }

    pub fn might_contain(&self, key: &[u8]) -> bool {
        if self.bits.is_empty() {
            return true;
        }
        for i in 0..self.k {
            let h = hash_key(key, i as u64);
            let bit = (h as usize) % (self.bits.len() * 64);
            if self.bits[bit / 64] & (1u64 << (bit % 64)) == 0 {
                return false;
            }
        }
        true
    }
}

fn hash_key(key: &[u8], seed: u64) -> u64 {
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    key.hash(&mut h);
    h.finish()
}

fn optimal_bits(n: usize, p: f64) -> usize {
    let n = n.max(1) as f64;
    let p = p.clamp(0.0001, 0.5);
    (-(n * p.ln()) / (2.0_f64.ln().powi(2))).ceil() as usize
}

fn optimal_k(m: usize, n: usize) -> usize {
    let n = n.max(1) as f64;
    ((m as f64 / n) * 2.0_f64.ln()).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bloom_no_false_negatives() {
        let mut b = BloomFilter::with_capacity(100, 0.01);
        b.insert(b"key1");
        assert!(b.might_contain(b"key1"));
        assert!(!b.might_contain(b"missing"));
    }
}
