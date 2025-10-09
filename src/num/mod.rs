//! Stuff for working with number stuff

#![allow(unused)]

mod value;
pub use value::*;
mod int;
pub use int::*;
mod constraint;
pub use constraint::*;

/// Errors encountered in these libraries
#[derive(Debug)]
pub enum Error {
    /// Failed to parse a signed value
    ParseSigned(std::num::ParseIntError),

    /// Failed to parse an unsigned value
    ParseUnsigned(std::num::ParseIntError),

    /// Integer truncation happened when converting a `u64` to a `usize`
    TooBig,

    /// Failed to parse a floating point value
    ParseFloat(std::num::ParseFloatError),

    /// Invalid constraint
    InvalidConstraint,

    /// An invalid expression was used
    ///
    /// Currently we just support add, sub, mul, and div. No spaces. Numbers
    /// can be any base (with the correct override)
    InvalidExpression,
}
