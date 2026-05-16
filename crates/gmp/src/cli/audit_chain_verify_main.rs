use clap::Parser;
use std::path::PathBuf;

mod audit_chain_verify;

use audit_chain_verify::{run_verification, write_report, VerifyMode};

#[derive(Parser, Debug)]
#[command(name = "audit-chain-verify")]
#[command(about = "Audit Chain Verification Tool for SQLRustGo GMP")]
struct Args {
    #[arg(long, default_value = "quick")]
    mode: String,

    #[arg(long)]
    chain_dir: PathBuf,

    #[arg(long)]
    seq: Option<u64>,

    #[arg(long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let mode = match args.mode.to_lowercase().as_str() {
        "quick" => VerifyMode::Quick,
        "full" => VerifyMode::Full,
        "entry" => VerifyMode::Entry,
        "incremental" => VerifyMode::Incremental,
        _ => {
            eprintln!(
                "Invalid mode: {}. Use quick, full, entry, or incremental.",
                args.mode
            );
            std::process::exit(1);
        }
    };

    match run_verification(&mode, &args.chain_dir, args.seq) {
        Ok(report) => {
            if let Err(e) = write_report(&report, args.output.as_ref()) {
                eprintln!("Failed to write report: {}", e);
                std::process::exit(1);
            }

            if report.results.passed {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Verification failed: {}", e);
            std::process::exit(1);
        }
    }
}
