//! `/proc/pid/maps` parser and stuff

use std::fmt;
use crate::{Error, Pid};

/// Memory permissions
#[derive(Debug, Clone)]
pub struct Permissions {
    /// Whether the memory is readable
    pub read: bool,

    /// Whether the memory is writeable
    pub write: bool,

    /// Whether the memory is executable
    pub execute: bool,

    /// Whether the memory shared or private (copy on write)
    pub shared: bool,
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

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}{}",
            if self.read    { 'r' } else { '-' },
            if self.write   { 'w' } else { '-' },
            if self.execute { 'x' } else { '-' },
            if self.shared  { 's' } else { 'p' })
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

    /// The raw memory of this region
    pub memory: Option<Vec<u8>>,
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

        // If the length is 0, skip it
        if addr.start == addr.end { return None; }

        // Get the persmissions
        let perms = Permissions::from_str(splits.next()?)?;

        // Get the path if there's any at all
        let path = splits.nth(3).map(str::to_string);

        // Return the parsed memory region
        Some(Self { addr, perms, path, memory: None })
    }

    /// Get the address range for this region
    pub fn addr(&self) -> &core::ops::Range<usize> {
        &self.addr
    }

    /// Get the permissions for this region
    pub fn perms(&self) -> &Permissions {
        &self.perms
    }

    /// Get the backing file path or pseudo-path for this region
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:<014X?} | 0x{:<9X} {} {}",
            self.addr,
            self.addr.end - self.addr.start,
            self.perms,
            match &self.path {
                None => "",
                Some(p) => &p,
            }
        )
    }
}

/// Regions in `/proc/pid/maps`
#[derive(Debug, Clone)]
pub struct Maps(pub Vec<Region>);

impl Maps {
    /// Get path to the maps file
    fn path(pid: Pid) -> String {
        format!("/proc/{}/maps", pid.0)
    }

    /// Check whether the maps file is accessible
    pub fn accessible(pid: Pid) -> crate::Result<()> {
        let _ = std::fs::File::open(Self::path(pid)).map_err(Error::Io)?;
        Ok(())
    }

    /// Parse memory regions for `pid` and retain those passing the `filter`
    pub fn regions<F>(pid: Pid, filter: F) -> crate::Result<Self>
    where
        F: FnMut(&Region) -> bool,
    {
        let maps = std::fs::read_to_string(Self::path(pid)).map_err(Error::Io)?
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
