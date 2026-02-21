//! SQLRustGo - A Rust SQL-92 Database Implementation
//!
//! Interactive SQL REPL and command-line interface

use sqlrustgo::{init, parse, ExecutionResult};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    println!("╔════════════════════════════════════════════════╗");
    println!("║       SQLRustGo v1.0.0                        ║");
    println!("║  A Rust SQL-92 Database Implementation       ║");
    println!("╚════════════════════════════════════════════════╝");
    println!();
    println!("Type 'exit' or 'quit' to exit.");
    println!("Type '.help' for commands.");
    println!();

    init();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut engine = sqlrustgo::ExecutionEngine::new();

    while running.load(Ordering::SeqCst) {
        print!("sqlrustgo> ");
        if let Err(e) = io::stdout().flush() {
            eprintln!("Warning: failed to flush stdout: {}", e);
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }

                match process_input(input, &mut engine) {
                    Ok(Some(result)) => {
                        print_result(result);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
            }
        }
    }

    println!("\nGoodbye!");
}

/// Process user input
fn process_input(
    input: &str,
    engine: &mut sqlrustgo::ExecutionEngine,
) -> Result<Option<ExecutionResult>, String> {
    // Handle special commands
    if input.starts_with('.') {
        return handle_command(input);
    }

    // Handle exit
    if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
        std::process::exit(0);
    }

    // Execute SQL statement
    match parse(input) {
        Ok(statement) => match engine.execute(statement) {
            Ok(result) => Ok(Some(result)),
            Err(e) => Err(format!("Execution error: {}", e)),
        },
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}

/// Handle special commands
fn handle_command(input: &str) -> Result<Option<ExecutionResult>, String> {
    match input {
        ".help" => {
            println!("Available commands:");
            println!("  .help      Show this help message");
            println!("  .tables    List all tables");
            println!("  .schema    Show schema info");
            println!("  .exit      Exit the REPL");
            println!("  .quit      Exit the REPL");
            Ok(None)
        }
        ".tables" => {
            println!("Tables: (catalog not yet implemented)");
            Ok(None)
        }
        ".schema" => {
            println!("Schema: (catalog not yet implemented)");
            Ok(None)
        }
        ".exit" | ".quit" => {
            std::process::exit(0);
        }
        _ => Err("Unknown command. Type .help for available commands.".to_string()),
    }
}

/// Print execution result
fn print_result(result: ExecutionResult) {
    if result.rows.is_empty() {
        println!("OK, {} row(s) affected", result.rows_affected);
    } else {
        println!("{} row(s) in set", result.rows.len());
    }
}
