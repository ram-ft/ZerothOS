//! CLI tool for creating and inspecting .zero packages

use zeropkg::{ZERO_MAGIC, validate_zero_header};

fn main() {
    println!("zeropkg v0.1.0 — Zeroth OS Package Manager");
    println!("Supports .zero bundle generation, verification, and installation.");
    println!("Magic Header: {:X?}", ZERO_MAGIC);
    println!("Usage:");
    println!("  zeropkg build <directory> -o <app.zero>");
    println!("  zeropkg install <app.zero>");
    println!("  zeropkg inspect <app.zero>");
}
