//! The internal state of the memory scanner.
//!
//! Keeps track of all allocations and history and stuff

use crate::Pid;

#[derive(Debug)]
pub struct Scanner {
    /// The PID we want to scan
    pid: Pid,
}

impl Scanner {
    pub fn new(pid: Pid) -> Self {
        Self {
            pid
        }
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }
}
