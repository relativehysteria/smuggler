//! Utilities for handlers

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
