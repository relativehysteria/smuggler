//! Safe wrapper around [`process_vm_readv()`] for reading memory from a remote
//! process

use core::num::NonZero;
use crate::Pid;

/// Represents a contiguous memory region for I/O
#[derive(Debug)]
#[repr(C)]
pub struct IoVec {
    /// Start address of the region
    pub base: u64,

    /// Length of the memory region in byte
    pub len: NonZero<usize>,
}

impl IoVec {
    /// Constructs a new `IoVec` with the given base address and length
    pub const fn new(base: u64, len: NonZero<usize>) -> Self {
        Self { base, len }
    }
}

unsafe extern "C" {
    /// The raw `process_vm_readv()` syscall
    pub fn process_vm_readv(
        pid:          Pid,
        local:        *const IoVec,
        local_count:  usize,
        remote:       *const IoVec,
        remote_count: usize,
        flags:        usize,
    ) -> isize;
}

/// Attempts to read `len` of data at `addr` from a remote process
pub fn read(pid: Pid, addr: u64, len: NonZero<usize>) -> Option<Vec<u8>> {
    // Create the remote iovec out of the arguments
    let remote = IoVec::new(addr, len);

    // Create the backing vector for this memory
    let mut backing: Vec<u8> = Vec::with_capacity(remote.len.into());

    // Create the local iovec
    let local = IoVec::new(backing.as_ptr() as u64, remote.len);

    // Do the read
    let read = unsafe {
        process_vm_readv(pid,
            core::ptr::addr_of!(local), 1,
            core::ptr::addr_of!(remote), 1, 0)
    };

    if read >= 0 && read as usize == remote.len.into() {
        unsafe { backing.set_len(remote.len.into()); }
        Some(backing)
    } else {
        None
    }
}

/// Reads memory from the specified `remote` iovecs into local buffers.
///
/// Each remote iovec maps 1:1 to a local buffer of the same size.
///
/// If a region is invalid, it's skipped, and the function retries with
/// the remaining valid regions.
///
/// Will panic if no remote iovecs are provided or if more than
/// [`crate::IOV_MAX`] are provided.
pub fn read_vecs(pid: Pid, remote: &[IoVec]) -> Vec<Option<Vec<u8>>> {
    assert!(remote.len() > 0);
    assert!(*crate::IOV_MAX.get().unwrap() >= remote.len());

    // Allocate local buffers matching each remote region
    let mut backing_vecs: Vec<Vec<u8>> = remote.iter()
        .map(|remote_iovec| Vec::with_capacity(remote_iovec.len.into()))
        .collect();

    // Then create local iovecs for each of those vectors
    let local: Vec<IoVec> = backing_vecs.iter_mut()
        .map(|vec| (vec.as_mut_ptr(), NonZero::new(vec.capacity())))
        .map(|(ptr, cap)| IoVec::new(ptr as u64, cap.unwrap()))
        .collect();

    // NOTE: If the first remote iovec is invalid, `process_vm_readv` returns
    // `EFAULT` immediately. If a later one is invalid, it returns the number
    // of bytes read so far. We retry until all regions are processed.

    // Get the total bytes that have yet to be read
    let mut to_read: usize = backing_vecs.iter().map(Vec::capacity).sum();

    // Index to track valid iovectors
    let mut current_idx = 0;

    'read: loop {
        // Attempt to read the memory into the local buffers
        let just_read: isize = unsafe {
            process_vm_readv(
                pid,
                local[current_idx..].as_ptr(),
                local.len() - current_idx,
                remote[current_idx..].as_ptr(),
                remote.len() - current_idx,
                0,
            )
        };

        // If the first iovec is invalid, skip it
        if just_read < 0 {
            to_read -= backing_vecs[current_idx].capacity();
            current_idx += 1;

            // If this iovec is also the last, stop, otherwise continue reading
            if current_idx == remote.len() { break; } else { continue; }
        }

        // Cast just_read to usize as this is now guaranteed positive due to the
        // check above
        let mut just_read = just_read as usize;

        // We got a read!
        for vec in backing_vecs[current_idx..].iter_mut() {
            // Take note of how many more bytes we have to read
            let cap = vec.capacity();
            to_read -= cap;

            // Update the current index to the iovecs for the next call
            current_idx += 1;

            // If there's no more bytes to read, this is the last iovec
            if to_read == 0 {
                // If we read enough to fill it, set its length. Otherwise this
                // is an incomplete read so the iovec is invalid and skipped
                if just_read == cap { unsafe { vec.set_len(cap); } }
                break 'read;
            }

            // There's still more shit to read

            // If we read enough to fill this vector, mark it as such; go next
            if just_read >= cap {
                unsafe { vec.set_len(cap); }
                just_read -= cap;
                continue;
            }

            // This iovec caused an incomplete read. `current_idx` already
            // points past it, so it will be skipped on the next call
            break;
        }
    }

    // Get rid of partially read vectors
    backing_vecs.into_iter()
        .map(|v| (!v.is_empty()).then_some(v))
        .collect()
}
