//! HashJoin Executor
//!
//! Implements hash-based inner join for the Volcano model.

use crate::executor::executor::{Executor, RecordBatch};
use crate::types::{SqlResult, Value};
use std::collections::HashMap;

/// HashJoinExecutor performs inner hash join between two inputs
pub struct HashJoinExecutor {
    left: Box<dyn Executor>,
    right: Box<dyn Executor>,
    left_key_indices: Vec<usize>,
    right_key_indices: Vec<usize>,
    hash_table: HashMap<String, Vec<Vec<Value>>>,
    phase: JoinPhase,
    output_columns: Vec<String>,
    probe_index: usize,
    right_buffer: Vec<Vec<Value>>,
    buffer_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum JoinPhase {
    Build,
    Probe,
    Done,
}

impl HashJoinExecutor {
    pub fn new(
        left: Box<dyn Executor>,
        right: Box<dyn Executor>,
        left_key_indices: Vec<usize>,
        right_key_indices: Vec<usize>,
        output_columns: Vec<String>,
    ) -> Self {
        Self {
            left,
            right,
            left_key_indices,
            right_key_indices,
            hash_table: HashMap::new(),
            phase: JoinPhase::Build,
            output_columns,
            probe_index: 0,
            right_buffer: vec![],
            buffer_index: 0,
        }
    }

    fn key_to_string(&self, row: &[Value], key_indices: &[usize]) -> String {
        key_indices.iter()
            .filter_map(|&idx| row.get(idx))
            .map(|v| format!("{:?}", v))
            .collect::<Vec<_>>()
            .join("|")
    }
}

impl Executor for HashJoinExecutor {
    fn next(&mut self) -> SqlResult<Option<RecordBatch>> {
        match self.phase {
            JoinPhase::Build => {
                loop {
                    let batch = match self.left.next()? {
                        Some(b) => b,
                        None => break,
                    };
                    for row in &batch.rows {
                        let key = self.key_to_string(row, &self.left_key_indices);
                        self.hash_table.entry(key).or_insert_with(Vec::new).push(row.clone());
                    }
                }
                self.phase = JoinPhase::Probe;
                self.right_buffer.clear();
                return self.next();
            }
            JoinPhase::Probe => {
                // Try to get next right row
                if self.buffer_index >= self.right_buffer.len() {
                    // Need more right rows
                    match self.right.next()? {
                        Some(batch) => {
                            self.right_buffer = batch.rows;
                            self.buffer_index = 0;
                        }
                        None => {
                            self.phase = JoinPhase::Done;
                            return Ok(None);
                        }
                    }
                }

                if self.buffer_index < self.right_buffer.len() {
                    let right_row = self.right_buffer[self.buffer_index].clone();
                    self.buffer_index += 1;

                    let key = self.key_to_string(&right_row, &self.right_key_indices);
                    if let Some(matches) = self.hash_table.get(&key) {
                        let mut joined_rows = Vec::new();
                        for left_row in matches {
                            let mut joined = left_row.clone();
                            joined.extend(right_row.clone());
                            joined_rows.push(joined);
                        }
                        return Ok(Some(RecordBatch::new(self.output_columns.clone(), joined_rows)));
                    }
                }
                
                // No match, continue probing
                Ok(Some(RecordBatch::empty()))
            }
            JoinPhase::Done => Ok(None),
        }
    }

    fn schema(&self) -> &[String] { &self.output_columns }
    fn init(&mut self) -> SqlResult<()> {
        self.hash_table.clear();
        self.phase = JoinPhase::Build;
        self.probe_index = 0;
        self.right_buffer.clear();
        self.buffer_index = 0;
        self.left.init()?;
        self.right.init()
    }
}
