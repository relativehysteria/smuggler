//! The internal state of the memory scanner.
//!
//! Keeps track of all allocations and history and stuff

use std::sync::OnceLock;
use crate::Pid;

/// The amount of memory to read in a single go when scanning
pub const CHUNK_SIZE: usize = 1024 * 1024 * 1024;

/// The maximum number of iovecs the `process_vm_readv()` syscall can handle
pub static IOV_MAX: OnceLock<usize> = OnceLock::new();

unsafe extern "C" {
    /// The raw `sysconf()` syscall
    pub fn sysconf(name: i32) -> isize;
}

#[derive(Debug)]
pub struct Scanner {
    /// The PID we want to scan
    pid: Pid,

    /// History of results
    pub results: Vec<Vec<u64>>,
}

impl Scanner {
    /// Create a new scanner
    pub fn new(pid: Pid) -> Self {
        // If we haven't yet set `IOV_MAX`, do so
        if IOV_MAX.get().is_none() {
            // This should be stable on x86-64
            const _SC_IOV_MAX: i32 = 60;

            let val = unsafe { sysconf(_SC_IOV_MAX) };
            let _ = IOV_MAX.set(usize::try_from(val).unwrap());
        }

        Self { pid, results: Vec::new(), }
    }

    /// Get the PID of the scanned process
    pub fn pid(&self) -> Pid {
        self.pid
    }
}
