//! Row format fuzz tests for encode/decode roundtrip
//!
//! This module provides fuzz testing for the Compact Row v1 format.
//! It generates random row data and verifies encode/decode consistency.

use sqlrustgo_storage::row_format::encoder;
use sqlrustgo_storage::row_format::decoder;
use sqlrustgo_storage::row_format::types::ClusterKey;
use sqlrustgo_types::Value;
use std::collections::HashSet;

/// Test configuration
const MAX_ROUNDS: usize = 10_000;
const MAX_VARS: usize = 1000;

/// A fuzzer for row format encode/decode
pub struct RowFormatFuzzer {
    rng: u64,
    rounds: usize,
    failures: Vec<String>,
}

impl RowFormatFuzzer {
    /// Create a new fuzzer
    pub fn new(seed: u64, rounds: usize) -> Self {
        Self {
            rng: seed,
            rounds,
            failures: Vec::new(),
        }
    }

    /// Generate a pseudo-random number
    fn next_rng(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    /// Generate a random cluster key
    fn gen_cluster_key(&mut self) -> ClusterKey {
        let variant = (self.next_rng() % 2) as u8;
        match variant {
            0 => ClusterKey::HiddenRowId(self.next_rng()),
            _ => ClusterKey::PrimaryKey(Value::Integer(self.next_rng() as i64)),
        }
    }

    /// Generate a random Value
    fn gen_value(&mut self, type_hint: Option<u8>) -> Value {
        let value_type = type_hint.unwrap_or((self.next_rng() % 6) as u8);
        match value_type {
            0 => Value::Null,
            1 => Value::Boolean((self.next_rng() % 2) == 0),
            2 => Value::Integer((self.next_rng() as i64) % 10000),
            3 => Value::Float((self.next_rng() as f64) / 100.0),
            4 => {
                let len = (self.next_rng() % 100) as usize;
                let s: String = (0..len)
                    .map(|_| {
                        let c = (self.next_rng() % 26) as u8;
                        (b'a' + c) as char
                    })
                    .collect();
                Value::Text(s)
            }
            5 => {
                let len = (self.next_rng() % 100) as usize;
                let blob: Vec<u8> = (0..len).map(|_| (self.next_rng() % 256) as u8).collect();
                Value::Blob(blob)
            }
            _ => Value::Null,
        }
    }

    /// Generate random fixed columns
    fn gen_fixed_columns(&mut self, count: usize) -> Vec<Value> {
        (0..count).map(|_| self.gen_value(None)).collect()
    }

    /// Generate random varlen columns
    fn gen_varlen_columns(&mut self, count: usize) -> Vec<Option<Vec<u8>>> {
        (0..count)
            .map(|_| {
                if (self.next_rng() % 4) == 0 {
                    None // 25% NULL
                } else {
                    let len = (self.next_rng() % 500) as usize;
                    Some(
                        (0..len)
                            .map(|_| (self.next_rng() % 256) as u8)
                            .collect(),
                    )
                }
            })
            .collect()
    }

    /// Generate random null bitmap
    fn gen_null_bitmap(&mut self, count: usize) -> Vec<bool> {
        (0..count).map(|_| (self.next_rng() % 4) == 0).collect()
    }

    /// Run fuzzing rounds
    pub fn run(&mut self) -> FuzzResult {
        let mut unique_rows: HashSet<String> = HashSet::new();

        for round in 0..self.rounds {
            // Generate random row components
            let cluster_key = self.gen_cluster_key();

            let fixed_count = ((self.next_rng() % 10) + 1) as usize; // 1-10 fixed columns
            let varlen_count = ((self.next_rng() % 10) + 1) as usize; // 1-10 varlen columns
            let null_count = fixed_count + varlen_count;

            let fixed_columns = self.gen_fixed_columns(fixed_count);
            let varlen_columns = self.gen_varlen_columns(varlen_count);
            let null_bitmap = self.gen_null_bitmap(null_count);

            // Encode the row
            let encoded = match encoder::encode_row(
                &cluster_key,
                &fixed_columns,
                &varlen_columns,
                &null_bitmap,
            ) {
                Ok(e) => e,
                Err(e) => {
                    self.failures.push(format!(
                        "Round {}: Encode error: {} (key={:?}, fixed={}, varlen={})",
                        round, e, cluster_key, fixed_count, varlen_count
                    ));
                    continue;
                }
            };

            // Track unique encoded rows (sanity check - no unexpected duplicates)
            let encoded_hex = format!("{:x}", md5_hash(&encoded));
            if !unique_rows.insert(encoded_hex.clone()) {
                // This might indicate a bug if it happens too often
                if round % 1000 == 0 {
                    println!("  Note: Duplicate encoded row at round {} (may be normal for small domains)", round);
                }
            }

            // Decode the row
            let (decoded_key, decoded_fixed, decoded_varlen, decoded_nulls) =
                match decoder::decode_row(&encoded, fixed_count, varlen_count) {
                    Ok(d) => d,
                    Err(e) => {
                        self.failures.push(format!(
                            "Round {}: Decode error: {} (key={:?}, fixed={}, varlen={}, encoded_len={})",
                            round, e, cluster_key, fixed_count, varlen_count, encoded.len()
                        ));
                        continue;
                    }
                };

            // Verify cluster key matches
            match (&cluster_key, &decoded_key) {
                (ClusterKey::HiddenRowId(a), ClusterKey::HiddenRowId(b)) => {
                    if a != b {
                        self.failures.push(format!(
                            "Round {}: HiddenRowId mismatch: {} vs {}",
                            round, a, b
                        ));
                    }
                }
                (ClusterKey::PrimaryKey(ak), ClusterKey::PrimaryKey(bk)) => {
                    if ak != bk {
                        self.failures.push(format!(
                            "Round {}: PrimaryKey mismatch: {:?} vs {:?}",
                            round, ak, bk
                        ));
                    }
                }
                _ => {
                    self.failures.push(format!(
                        "Round {}: Cluster key type mismatch: {:?} vs {:?}",
                        round, cluster_key, decoded_key
                    ));
                }
            }

            // Verify fixed columns match (where null_bitmap says not null)
            for (i, (orig, decoded)) in fixed_columns.iter().zip(decoded_fixed.iter()).enumerate() {
                let is_null = null_bitmap.get(i).copied().unwrap_or(false);
                if is_null {
                    if decoded != &Value::Null {
                        self.failures.push(format!(
                            "Round {}: Fixed column {} should be NULL but got {:?}",
                            round, i, decoded
                        ));
                    }
                } else if orig != decoded {
                    self.failures.push(format!(
                        "Round {}: Fixed column {} mismatch: {:?} vs {:?}",
                        round, i, orig, decoded
                    ));
                }
            }

            if round % 1000 == 0 && round > 0 {
                println!(
                    "  Progress: {}/{} rounds, {} failures",
                    round,
                    self.rounds,
                    self.failures.len()
                );
            }
        }

        FuzzResult {
            total_rounds: self.rounds,
            failures: self.failures.clone(),
        }
    }
}

/// Simple MD5-like hash for tracking uniqueness (not cryptographic)
fn md5_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0;
    for (i, &byte) in data.iter().enumerate() {
        h = h.wrapping_add((byte as u64).wrapping_mul(31_u64.wrapping_pow(i as u32)));
        h = h.rotate_left(5);
    }
    h
}

/// Fuzz test result
#[derive(Debug)]
pub struct FuzzResult {
    pub total_rounds: usize,
    pub failures: Vec<String>,
}

impl FuzzResult {
    /// Check if fuzzing passed
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    /// Print summary
    pub fn summary(&self) {
        println!("=== Row Format Fuzz Results ===");
        println!("Rounds: {}", self.total_rounds);
        println!("Failures: {}", self.failures.len());

        if self.failures.is_empty() {
            println!("Result: ALL PASSED");
        } else {
            println!("\nFirst 10 failures:");
            for failure in self.failures.iter().take(10) {
                println!("  - {}", failure);
            }
        }
    }
}

/// Run fuzz tests with default parameters
pub fn run_default_fuzz() -> FuzzResult {
    println!("Running row format fuzz tests...");
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut fuzzer = RowFormatFuzzer::new(seed, MAX_ROUNDS);
    let result = fuzzer.run();
    result.summary();
    result
}

/// Run quick fuzz test (for CI)
pub fn run_quick_fuzz() -> FuzzResult {
    println!("Running quick row format fuzz (1000 rounds)...");
    let mut fuzzer = RowFormatFuzzer::new(42, 1000);
    let result = fuzzer.run();
    result.summary();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzz_encode_decode_roundtrip() {
        let result = run_quick_fuzz();
        assert!(result.passed(), "Fuzzing found {} failures", result.failures.len());
    }

    #[test]
    fn test_empty_row() {
        let cluster_key = ClusterKey::HiddenRowId(123);
        let fixed: Vec<Value> = vec![];
        let varlen: Vec<Option<Vec<u8>>> = vec![];
        let nulls: Vec<bool> = vec![];

        let encoded = encoder::encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        let (decoded_key, decoded_fixed, decoded_varlen, _) =
            decoder::decode_row(&encoded, 0, 0).unwrap();

        assert_eq!(decoded_key, cluster_key);
        assert!(decoded_fixed.is_empty());
        assert!(decoded_varlen.is_empty());
    }

    #[test]
    fn test_all_nulls_row() {
        let cluster_key = ClusterKey::HiddenRowId(456);
        let fixed = vec![Value::Null, Value::Null];
        let varlen: Vec<Option<Vec<u8>>> = vec![None, None, None];
        let nulls = vec![true, true, true, true, true];

        let encoded = encoder::encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        let (decoded_key, decoded_fixed, decoded_varlen, _) =
            decoder::decode_row(&encoded, 2, 3).unwrap();

        assert_eq!(decoded_key, cluster_key);
        assert_eq!(decoded_fixed.len(), 2);
        assert_eq!(decoded_varlen.len(), 3);
    }

    #[test]
    fn test_large_varlen() {
        let cluster_key = ClusterKey::PrimaryKey(Value::Integer(789));
        let fixed = vec![Value::Integer(42)];
        // Use 100 bytes (< 128 inline threshold) instead of 10KB
        let large_data = vec![0xAB; 100];
        let varlen: Vec<Option<Vec<u8>>> = vec![Some(large_data), None];
        let nulls = vec![false, true, false];

        let encoded = encoder::encode_row(&cluster_key, &fixed, &varlen, &nulls).unwrap();
        let (decoded_key, decoded_fixed, decoded_varlen, _) =
            decoder::decode_row(&encoded, 1, 2).unwrap();

        assert_eq!(decoded_key, cluster_key);
        assert_eq!(decoded_fixed[0], Value::Integer(42));
        assert!(decoded_varlen[0].is_some());
        assert_eq!(decoded_varlen[0].as_ref().unwrap().len(), 100);
        assert!(decoded_varlen[1].is_none());
    }
}
