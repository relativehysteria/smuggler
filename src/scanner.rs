//! The internal state of the memory scanner.
//!
//! Keeps track of all allocations and history and stuff

use crate::Pid;

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
