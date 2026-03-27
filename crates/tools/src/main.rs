//! SQLRustGo Tools
//!
//! Collection of utility tools for SQLRustGo database.

mod catalog_check;

fn main() {
    if let Err(e) = catalog_check::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
