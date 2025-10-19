#![feature(iter_intersperse)]

use core::num::NonZero;

#[macro_use] pub mod commands;
#[macro_use] pub mod num;
pub mod cli;
pub mod remote;
pub mod proc_maps;
pub use proc_maps::Maps;
mod scanner;
pub use scanner::*;

/// Wrapper around [`std::result::Result`] for this application
pub type Result<T> = std::result::Result<T, Error>;

/// Generic error type for this application
#[derive(Debug)]
pub enum Error {
    /// The specified PID is not a number
    PidNotInt(std::num::ParseIntError),

    /// The specified PID is a zero, which is invalid on linux
    ZeroPid,

    /// An error returned by [`rustyline`]
    Cli(rustyline::error::ReadlineError),

    /// A generic I/O error
    Io(std::io::Error),

    /// Error returned by the `num` module
    Num(crate::num::Error),
}

impl From<num::Error> for Error {
    fn from(val: num::Error) -> Self {
        Self::Num(val)
    }
}

/// System process ID
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Pid(pub NonZero<usize>);

impl TryFrom<&str> for Pid {
    type Error = Error;

    fn try_from(value: &str) -> crate::Result<Self> {
        let val = usize::from_str_radix(value, 10).map_err(Error::PidNotInt)?;
        NonZero::new(val).ok_or(Error::ZeroPid).map(|v| Self(v))
    }
}
