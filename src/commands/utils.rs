//! Utilities for handlers

use crate::Scanner;
use crate::num::{Constraint, Value};

/// Helper to extract a `T` from `arg` that generates nice error messages
pub fn parse_arg<T: crate::num::ParseNumber>(arg: Option<&&str>, name: &str)
        -> Result<T, String> {
    arg
        .ok_or_else(|| format!("{} missing", name))
        .and_then(|s| crate::num::parse::<T>(&s)
            .map_err(|e| format!("{} not a valid number: {:?}", name, e)))
}

/// Helper to extract a value from `arg` that generates nice error messages
pub fn parse_value(arg: Option<&&str>) -> Result<crate::num::Value, String> {
    arg.and_then(|arg| arg.chars().nth(1))
        .map(crate::num::Value::default_from_letter)
        .ok_or("Missing or invalid type specifier".to_string())
}

/// Helper to extract constraints from `args` that generates nice error messages
pub fn parse_constraints(args: &[&str], value: Value)
        -> Result<Vec<Constraint>, String> {
    if args.is_empty() { return Err("Constraints missing".to_string()); }

    args.iter()
        .map(|&c| Constraint::from_str_value(c, Some(value))
            .map_err(|e| format!("Couldn't parse constraints: {:?}", e)))
        .collect::<Result<Vec<Constraint>, String>>()
}

/// Print the results of a scan to the screen and save them in the scanner
pub fn print_and_save_results(s: &mut Scanner, results: Vec<u64>) {
    // Print the results
    if results.is_empty() {
        println!("No results.");
    } else {
        if results.len() > 10 {
            println!("Found {} results.", results.len());
        } else if results.len() == 1 {
            println!("Found 1 match at:");
            println!("  0x{:X}", results[0])
        } else {
            println!("Found {:?} results at:", results.len());
            for addr in results.iter() {
                println!("  0x{:X}", addr);
            }
        }

        // Save the results
        s.results = results;
    }
}
