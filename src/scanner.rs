//! The internal state of the memory scanner.
//!
//! Keeps track of all allocations and history and stuff

use crate::Pid;

/// The amount of memory to read in a single go when scanning
pub const CHUNK_SIZE: usize = 1024 * 1024 * 1024;

#[derive(Debug)]
pub struct Scanner {
    /// The PID we want to scan
    pid: Pid,

    /// Results of a previous scan
    pub results: Vec<u64>,
}

impl Scanner {
    /// Create a new scanner
    pub fn new(pid: Pid) -> Self {
        Self { pid, results: Vec::new(), }
    }

    /// Get the PID of the scanned process
    pub fn pid(&self) -> Pid {
        self.pid
    }
}
