//! Utilities for handlers

use rayon::prelude::*;
use std::sync::{Arc, Mutex};
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

    // Retain only successfully read chunks
    let chunks: Vec<_> = batch.iter()
        .zip(memory.into_iter())
        .filter_map(|(iovec, mem)| mem.map(|m| (iovec, m)))
        .collect();

    // Shared matches vector with interior mutability
    let results = Arc::new(Mutex::new(Vec::new()));

    // Parallel iteration over chunks
    chunks.par_iter().for_each(|(iovec, mem)| {
        let mut local_results = Vec::new();

        // Local copy of the value
        let mut v = value;

        for (offset, chunk) in mem.chunks_exact(v.bytes()).enumerate() {
            v.from_le_bytes(chunk);

            // Check constraints
            if constraints.iter().all(|x| x.check(v)) {
                let abs = iovec.base + offset as u64 * v.bytes() as u64;
                local_results.push(abs);
            }
        }

        // Append to global results
        if !local_results.is_empty() {
            let mut guard = results.lock().unwrap();
            guard.extend(local_results);
        }
    });

    // Move collected results back into matches
    let mut guard = results.lock().unwrap();
    matches.extend(guard.drain(..));
}
