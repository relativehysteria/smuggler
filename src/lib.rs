pub mod proc_maps;
pub mod cli;
pub mod read_remote;

/// Wrapper for `Result`
pub type Result<T> = std::result::Result<T, Error>;

/// Generic error type for this application
#[derive(Debug)]
pub enum Error {
    InvalidPid(std::num::ParseIntError),
    CliError(rustyline::error::ReadlineError),
    IoError(std::io::Error),
}

/// System process ID
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Pid(pub usize);

impl TryFrom<&str> for Pid {
    type Error = Error;

    fn try_from(value: &str) -> crate::Result<Self> {
        Ok(Self(usize::from_str_radix(value, 10).map_err(Error::InvalidPid)?))
    }
}
