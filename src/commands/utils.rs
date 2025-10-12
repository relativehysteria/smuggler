//! Utilities for handlers

use crate::{Scanner, remote::IoVec};
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

/// Common utility function for scanning memory based on constraints
pub fn scan_batch(
    pid: crate::Pid,
    matches: &mut Vec<u64>,
    batch: &[IoVec],
    mut value: Value,
    constraints: &[Constraint],
) {
    // Read the memory
    let memory = crate::remote::read_vecs(pid, &batch);

    // Retain only those chunks of memory that have been successfully read
    let chunks = batch.iter().zip(memory.into_iter())
        .filter(|(_, mem)| mem.is_some())
        .map(|(iovec, mem)| (iovec, mem.unwrap()));

    // Go through each region and scan
    for (iovec, mem) in chunks {
        // Go through the region in chunks
        for (offset, chunk) in mem.chunks_exact(value.bytes()).enumerate() {
            // Update the value
            value.from_le_bytes(chunk);

            // Check that constraints match and if they do, save the address
            if constraints.iter().all(|x| x.check(value)) {
                let abs = iovec.base + offset as u64 * value.bytes() as u64;
                matches.push(abs);
            }
        }
    }
}
