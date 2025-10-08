use crate::Pid;
use crate::proc_maps::Region;

/// Memory region for I/O operations
#[repr(C)]
pub struct IoVec {
    /// Starting address
    pub base: usize,

    /// Size of the memory region
    pub len: usize,
}

impl IoVec {
    /// Constructs a new IoVec
    pub fn new(base: usize, len: usize) -> Self {
        Self { base, len }
    }
}

unsafe extern "C" {
    fn process_vm_readv(
        pid:          Pid,
        local:        *const IoVec,
        local_count:  usize,
        remote:       *const IoVec,
        remote_count: usize,
        flags:        usize,
    ) -> isize;
}

/// Populate the memory `regions` with their raw memory bytes
pub fn populate_regions(pid: Pid, regions: &mut [Region]) {
    // Create remote iovecs for the regions
    let remote: Vec<IoVec> = regions.iter()
        .map(|r| IoVec::new(r.addr().start, r.addr().end - r.addr().start))
        .collect();

    // Read the memory regions
    let memory_regions = remote_readv(pid, &remote);

    // Populate the memory pointers in the regions
    regions.iter_mut()
        .zip(memory_regions.into_iter())
        .for_each(|(region, memory)| region.memory = memory);
}

/// Read the following iovectors into memory
///
/// The mapping between local and remote iovecs is 1:1. That is, each remote
/// iovec will be written to a same sized local iovec
///
/// If a memory region is invalid, it is completely skipped and another syscall
/// is issued for the remaining valid regions
fn remote_readv(pid: Pid, remote: &[IoVec]) -> Vec<Option<Vec<u8>>> {
    assert!(remote.len() > 0);

    // First, create the vectors backing up the local memory
    let mut backing_vecs: Vec<Vec<u8>> = remote.iter()
        .map(|remote_iovec| Vec::with_capacity(remote_iovec.len))
        .collect();

    // Then create local iovecs for each of those vectors
    let local: Vec<IoVec> = backing_vecs.iter_mut()
        .map(|vec| IoVec::new(vec.as_mut_ptr() as usize, vec.capacity()))
        .collect();

    // XXX: FROM WHAT I UNDERSTAND FROM THE IMPLICIT MANPAGE AND REFUSE TO TEST:
    // If the first remote iovec is immediately invalid, EFAULT is returned
    // instead of bytes read. If any other iovec becomes invalid during the
    // call, bytes read is returned instead. So simply keep reading until we
    // reach the last remote iovec

    // Get the total bytes that have yet to be read
    let mut to_read = backing_vecs.iter()
        .fold(0, |acc, vec| acc + vec.capacity());

    // Index to track valid iovectors
    let mut current_idx = 0;

    'read: loop {
        // Attempt to read the memory into the local buffers
        let mut just_read = unsafe {
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
            if current_idx == remote.len() {
                break;
            } else {
                continue;
            }
        }

        // Cast just_read to usize as this is now guaranteed positive
        let mut just_read = just_read as usize;

        // We got a read!
        for (i, mut vec) in backing_vecs[current_idx..].iter_mut().enumerate() {
            // Take note of how many more bytes we have to read
            let cap = vec.capacity();
            to_read -= cap;

            // If there's no more bytes to read, this is the last iovec
            if to_read == 0 {
                // If we read enough to fill it, set its length. Otherwise this
                // is an incomplete read so the iovec is invalid and skipped
                if just_read == cap {
                    unsafe { vec.set_len(cap); }
                }
                break 'read;
            }

            // There's still more shit to read

            // If we read enough to fill this vector, mark it as such; go next
            if just_read >= cap {
                unsafe { vec.set_len(cap); }
                just_read -= cap;
                continue;
            }

            // Incomplete read. simply skip this iovec and go to the next one.
            // By keeping its length at 0 we'll be able to later drop it and
            // replace it with `None`
            current_idx = i + 1;
            break;
        }
    }

    // Get rid of partially read vectors
    backing_vecs.into_iter()
        .map(|vec| if vec.len() == 0 { None } else { Some(vec) })
        .collect()
}
