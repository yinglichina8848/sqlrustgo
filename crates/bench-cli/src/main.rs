use clap::Parser;
use std::path::Path;

mod cli;
mod commands;
mod metrics;
mod reporter;

use commands::{custom, oltp, tpch};

#[derive(Parser, Debug)]
#[command(name = "benchmark")]
#[command(about = "SQLRustGo Benchmark CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Tpch(cli::TpchArgs),
    Oltp(cli::OltpArgs),
    Custom(cli::CustomArgs),
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Tpch(tpch_args) => {
            let output = tpch_args.output.clone();
            let result = tpch::run(tpch_args);
            if let Some(output_path) = output {
                result.save(Path::new(&output_path)).unwrap();
                println!("Results saved to: {}", output_path);
            } else {
                result.print_json();
            }
        }
        Command::Oltp(oltp_args) => {
            let output = oltp_args.output.clone();
            let result = oltp::run(oltp_args);
            if let Some(output_path) = output {
                result.save(Path::new(&output_path)).unwrap();
                println!("Results saved to: {}", output_path);
            } else {
                result.print_json();
            }
        }
        Command::Custom(custom_args) => {
            let output = custom_args.output.clone();
            let result = custom::run(custom_args);
            if let Some(output_path) = output {
                result.save(Path::new(&output_path)).unwrap();
                println!("Results saved to: {}", output_path);
            } else {
                result.print_json();
            }
        }
    };
}
