use core::num::NonZero;

pub mod proc_maps;
pub mod cli;
pub mod read_remote;

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
    CliError(rustyline::error::ReadlineError),

    /// A generic I/O error
    IoError(std::io::Error),
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
