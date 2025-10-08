//! /proc/pid/maps parser and stuff

use crate::{Error, Pid};

/// Memory permissions
#[derive(Debug, Clone)]
struct Permissions {
    /// Whether the memory is readable
    read: bool,

    /// Whether the memory is writeable
    write: bool,

    /// Whether the memory is executable
    execute: bool,

    /// Whether the memory shared or private (copy on write)
    shared: bool,
}

impl Permissions {
    fn from_str(string: &str) -> Option<Self> {
        let mut chars = string.chars();

        let read    = chars.next()? == 'r';
        let write   = chars.next()? == 'w';
        let execute = chars.next()? == 'x';
        let shared  = chars.next()? == 's';

        Some(Self { read, write, execute, shared })
    }
}


/// A region of memory in `/proc/pid/maps`
#[derive(Debug, Clone)]
pub struct Region {
    /// The address range
    addr: core::ops::Range<usize>,

    /// The memory permissions
    perms: Permissions,

    /// The pathname of the file backing the mapping. Includes pseudo-paths
    /// (like `[stack]` etc.)
    path: Option<String>,
}

impl Region {
    /// Attempts to parse a line in `/proc/pid/maps`
    fn from_line(line: &str) -> Option<Self> {
        // Split the line on spaces
        let mut splits = line.split_whitespace();

        // Get the address range for this region
        let mut addr = splits.next()?.split('-');
        let addr = core::ops::Range {
            start: usize::from_str_radix(addr.next()?, 16).ok()?,
            end: usize::from_str_radix(addr.next()?, 16).ok()?,
        };

        // Get the persmissions
        let perms = Permissions::from_str(splits.next()?)?;

        // Get the path if there's any at all
        let path = splits.nth(3).map(str::to_string);

        // Return the parsed memory region
        Some(Self { addr, perms, path })
    }

    /// Get the address range for this region
    fn addr(&self) -> &core::ops::Range<usize> {
        &self.addr
    }

    /// Get the permissions for this region
    fn permissions(&self) -> &Permissions {
        &self.perms
    }

    /// Get the backing file path or pseudo-path for this region
    fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}


/// Regions in `/proc/pid/maps`
#[derive(Debug, Clone)]
pub struct Maps(pub Vec<Region>);

impl Maps {
    /// Parse memory regions for `pid` and retain those passing the `filter`
    pub fn regions<F>(pid: Pid, filter: F) -> crate::Result<Self>
    where
        F: FnMut(&Region) -> bool,
    {
        let file = format!("/proc/{}/maps", pid.0);
        let maps = std::fs::read_to_string(file).map_err(Error::IoError)?
            .lines()
            .filter_map(Region::from_line)
            .filter(filter)
            .collect();

        Ok(Self(maps))
    }

    /// Parse memory regions for `pid` and retain only the read-writeable ones
    pub fn rw_regions(pid: Pid) -> crate::Result<Self> {
        Self::regions(pid, |reg| reg.perms.read && reg.perms.write)
    }
}
