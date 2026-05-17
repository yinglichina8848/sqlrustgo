//! Crash Worker - WAL Crash Injection Test Tool
//!
//! Subprocess that writes WAL entries and exits, allowing parent test
//! to simulate crash by killing the process.
//!
//! Usage:
//!   crash-worker <wal-path> <mode> [args...]

use sqlrustgo_storage::wal::{WalEntry, WalEntryType, WalManager};
use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp as u64
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Create a WAL entry
fn make_entry(
    tx_id: u64,
    entry_type: WalEntryType,
    table_id: u64,
    key: &[u8],
    data: &[u8],
) -> WalEntry {
    WalEntry {
        tx_id,
        entry_type,
        table_id,
        key: Some(key.to_vec()),
        data: Some(data.to_vec()),
        lsn: 0,
        timestamp: current_timestamp(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: crash-worker <wal-path> <mode> [args...]");
        std::process::exit(1);
    }

    let wal_path = PathBuf::from(&args[1]);
    let mode = &args[2];

    match mode.as_str() {
        "write" => {
            // Write N entries and exit cleanly
            let count: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
            let wal = WalManager::new(wal_path);
            let mut writer = wal.get_writer().expect("Failed to get writer");

            for i in 0..count {
                let entry = make_entry(
                    i + 1,
                    WalEntryType::Insert,
                    1,
                    format!("key_{}", i).as_bytes(),
                    format!("value_{}", i).as_bytes(),
                );
                writer.append(&entry).expect("Failed to append");
            }
            writer.flush().expect("Failed to flush");
            println!("WROTE {} entries", count);
        }

        "write-flush" => {
            // Write N entries, flush, and exit
            let count: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
            let wal = WalManager::new(wal_path);
            let mut writer = wal.get_writer().expect("Failed to get writer");

            for i in 0..count {
                let entry = make_entry(
                    i + 1,
                    WalEntryType::Insert,
                    1,
                    format!("key_{}", i).as_bytes(),
                    format!("value_{}", i).as_bytes(),
                );
                writer.append(&entry).expect("Failed to append");
            }
            writer.flush().expect("Failed to flush");
            println!("WROTE-FLUSH {} entries", count);
        }

        "write-tx" => {
            // Write N transactions, each with Begin/Data/Commit
            let tx_count: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(3);
            let wal = WalManager::new(wal_path);
            let mut writer = wal.get_writer().expect("Failed to get writer");

            for tx in 0..tx_count {
                let tx_id = tx + 1;
                writer
                    .append(&WalEntry {
                        tx_id,
                        entry_type: WalEntryType::Begin,
                        table_id: 0,
                        key: None,
                        data: None,
                        lsn: 0,
                        timestamp: current_timestamp(),
                    })
                    .expect("Failed to append begin");

                writer
                    .append(&make_entry(
                        tx_id,
                        WalEntryType::Insert,
                        1,
                        format!("tx{}_key", tx_id).as_bytes(),
                        format!("tx{}_value", tx_id).as_bytes(),
                    ))
                    .expect("Failed to append insert");

                writer
                    .append(&WalEntry {
                        tx_id,
                        entry_type: WalEntryType::Commit,
                        table_id: 0,
                        key: None,
                        data: None,
                        lsn: 0,
                        timestamp: current_timestamp(),
                    })
                    .expect("Failed to append commit");
            }
            writer.flush().expect("Failed to flush");
            println!("WROTE {} transactions", tx_count);
        }

        "write-tx-rollback" => {
            // Write committed and rolled-back transactions
            let wal = WalManager::new(wal_path);
            let mut writer = wal.get_writer().expect("Failed to get writer");

            for (tx_id, et, k, d) in [
                (1u64, WalEntryType::Begin, None::<&str>, None::<&str>),
                (1, WalEntryType::Insert, Some("tx1_key"), Some("tx1_value")),
                (1, WalEntryType::Commit, None, None),
                (2, WalEntryType::Begin, None, None),
                (2, WalEntryType::Insert, Some("tx2_key"), Some("tx2_value")),
                (2, WalEntryType::Rollback, None, None),
            ] {
                writer
                    .append(&WalEntry {
                        tx_id,
                        entry_type: et,
                        table_id: if k.is_some() { 1 } else { 0 },
                        key: k.map(|s| s.as_bytes().to_vec()),
                        data: d.map(|s| s.as_bytes().to_vec()),
                        lsn: 0,
                        timestamp: current_timestamp(),
                    })
                    .expect("Failed to append");
            }
            writer.flush().expect("Failed to flush");
            println!("WROTE committed + rolled-back transactions");
        }

        "write-checkpoint" => {
            // Write transactions and a checkpoint
            let wal = WalManager::new(wal_path);
            let mut writer = wal.get_writer().expect("Failed to get writer");

            // TX 1: committed
            for (tx_id, et, k, d) in [
                (1u64, WalEntryType::Begin, None::<&str>, None::<&str>),
                (
                    1,
                    WalEntryType::Insert,
                    Some("ckpt_key1"),
                    Some("ckpt_val1"),
                ),
                (1, WalEntryType::Commit, None, None),
            ] {
                writer
                    .append(&WalEntry {
                        tx_id,
                        entry_type: et,
                        table_id: if k.is_some() { 1 } else { 0 },
                        key: k.map(|s| s.as_bytes().to_vec()),
                        data: d.map(|s| s.as_bytes().to_vec()),
                        lsn: 0,
                        timestamp: current_timestamp(),
                    })
                    .expect("Failed to append");
            }

            // Checkpoint
            writer
                .append(&WalEntry {
                    tx_id: 0,
                    entry_type: WalEntryType::Checkpoint,
                    table_id: 0,
                    key: None,
                    data: None,
                    lsn: 0,
                    timestamp: current_timestamp(),
                })
                .expect("Failed to append checkpoint");

            // TX 2: committed after checkpoint
            for (tx_id, et, k, d) in [
                (2u64, WalEntryType::Begin, None::<&str>, None::<&str>),
                (
                    2,
                    WalEntryType::Insert,
                    Some("ckpt_key2"),
                    Some("ckpt_val2"),
                ),
                (2, WalEntryType::Commit, None, None),
            ] {
                writer
                    .append(&WalEntry {
                        tx_id,
                        entry_type: et,
                        table_id: if k.is_some() { 1 } else { 0 },
                        key: k.map(|s| s.as_bytes().to_vec()),
                        data: d.map(|s| s.as_bytes().to_vec()),
                        lsn: 0,
                        timestamp: current_timestamp(),
                    })
                    .expect("Failed to append");
            }
            writer.flush().expect("Failed to flush");
            println!("WROTE 2 transactions + checkpoint");
        }

        "write-no-commit" => {
            // Write entries but never commit
            let count: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(3);
            let wal = WalManager::new(wal_path);
            let mut writer = wal.get_writer().expect("Failed to get writer");

            for i in 0..count {
                writer
                    .append(&WalEntry {
                        tx_id: 1,
                        entry_type: if i == 0 {
                            WalEntryType::Begin
                        } else {
                            WalEntryType::Insert
                        },
                        table_id: 1,
                        key: Some(format!("uncommitted_key_{}", i).into_bytes()),
                        data: Some(format!("uncommitted_value_{}", i).into_bytes()),
                        lsn: 0,
                        timestamp: current_timestamp(),
                    })
                    .expect("Failed to append");
            }
            writer.flush().expect("Failed to flush");
            println!("WROTE {} uncommitted entries (no commit)", count);
        }

        "recover-count" => {
            // Recover and print count of entries
            let wal = WalManager::new(wal_path);
            match wal.recover() {
                Ok(entries) => {
                    let committed = entries
                        .iter()
                        .filter(|e| e.entry_type == WalEntryType::Commit)
                        .count();
                    let rolled_back = entries
                        .iter()
                        .filter(|e| e.entry_type == WalEntryType::Rollback)
                        .count();
                    println!(
                        "TOTAL={} COMMITTED={} ROLLED_BACK={}",
                        entries.len(),
                        committed,
                        rolled_back
                    );
                }
                Err(e) => {
                    eprintln!("Recovery failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "recover-txids" => {
            // Recover and print transaction IDs that committed
            let wal = WalManager::new(wal_path);
            match wal.recover() {
                Ok(entries) => {
                    let committed_txs: Vec<u64> = entries
                        .iter()
                        .filter(|e| e.entry_type == WalEntryType::Commit)
                        .map(|e| e.tx_id)
                        .collect();
                    let rolled_back_txs: Vec<u64> = entries
                        .iter()
                        .filter(|e| e.entry_type == WalEntryType::Rollback)
                        .map(|e| e.tx_id)
                        .collect();
                    println!("COMMITTED_TXIDS={:?}", committed_txs);
                    println!("ROLLED_BACK_TXIDS={:?}", rolled_back_txs);
                }
                Err(e) => {
                    eprintln!("Recovery failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "check-integrity" => {
            // Check WAL file integrity (magic, version, etc.)
            use std::fs::File;
            use std::io::Read;

            let mut file = File::open(&wal_path).unwrap_or_else(|_| {
                println!("INTEGRITY=FILE_NOT_FOUND");
                std::process::exit(0);
            });
            let mut header = [0u8; 8];
            match file.read(&mut header) {
                Ok(0) | Err(_) => {
                    println!("INTEGRITY=EMPTY_OR_UNREADABLE");
                }
                Ok(_) => {
                    let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
                    let version = u16::from_le_bytes([header[4], header[5]]);

                    if magic == 0x57414C01 {
                        println!("INTEGRITY=VALID magic=0x{:08x} version={}", magic, version);
                    } else if magic == 0 {
                        println!("INTEGRITY=ZEROS_OR_EMPTY");
                    } else {
                        println!("INTEGRITY=CORRUPT magic=0x{:08x}", magic);
                    }
                }
            }
        }

        "file-size" => {
            // Print current WAL file size
            match std::fs::metadata(&wal_path) {
                Ok(meta) => println!("SIZE={}", meta.len()),
                Err(_) => println!("SIZE=0"),
            }
        }

        _ => {
            eprintln!("Unknown mode: {}", mode);
            std::process::exit(1);
        }
    }
}
