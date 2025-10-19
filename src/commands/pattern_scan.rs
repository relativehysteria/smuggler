use memchr::memmem;
use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["sp", "p", "pattern"],
r#"Search for non-overlapping occurrences of an IDA byte pattern

This command is slower than other scanning related commands due to it scanning
_all_ readable memory maps of the process and also because pattern scanning with
wildcards is just a slow process overall.

Nibble wildcards (e.g. `F?` or `?F`) are _not supported_."#,
r#"`<start_address> <end_address> <IDA_pattern>`
* `start_address` - Start searching from this address. If this is `0`, the
   search will start from the first readable memory region.
* `end_address` - Stop searching at this address. If this is `0`, the search
   will stop at the last readable memory region.
* `IDA_pattern` - The IDA byte pattern string to search for.
  For example, `48 65 6C 6C 6F ?? 20 ?? ?? 72 6C 64 ??`
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the start and end addresses
    let start = parse_arg::<u64>(args.get(1), "Start address")?;
    let end   = parse_arg::<u64>(args.get(2), "End address")?;

    // If end is undefined, default to the maximum address
    let end = if end == 0 { u64::MAX } else { end };

    // Get the anchors from the pattern string
    let anchors = Pattern::parse_scored_anchors(args.get(3..))?;

    // Get the memory map
    let mut maps = crate::proc_maps::Maps::r_regions(s.pid())
        .map_err(|e| format!("Couldn't parse memory map: {:?}", e))?;

    // Exclude kernel mappings and stuff
    maps.0.retain(|reg| reg.is_interesting());

    // Get the iovec batches
    let iovecs = maps.chunks(core::ops::Range { start, end });

    // Search for the string and save off the adresses where it's found
    let mut matches = Vec::new();

    for batch in iovecs.into_iter() {
        // Read the memory
        let memory = crate::remote::read_vecs(s.pid(), &batch);

        // Retain only those chunks of memory that have been successfully read
        let chunks = batch.iter().zip(memory.into_iter())
            .filter(|(_, mem)| mem.is_some())
            .map(|(iovec, mem)| (iovec, mem.unwrap()));

        // Go through each region and find the patterns that match
        for (iovec, mem) in chunks {
            let occurrences = anchors.find_pattern_iter(&mem)
                .map(|addr| iovec.base + addr as u64);
            matches.extend(occurrences);
        }
    }

    crate::commands::print_and_save_results(s, matches);

    Ok(())
}

/// An anchor of a pattern string
///
/// For example, in the pattern string `48 65 6C 6C 6F ?? 20 ?? ?? 72 6C 64 ??`
/// there are 3 anchors:
/// * `Anchor { offset: 0, bytes: [0x48, 0x65, 0x6C, 0x6C, 0x6F] }`
/// * `Anchor { offset: 6, bytes: [0x20] }`
/// * `Anchor { offset: 8, bytes: [0x72, 0x6C, 0x64] }`
#[derive(Debug)]
struct Anchor {
    /// The offset of this anchor into the pattern string
    offset: usize,

    /// Contiguous, known (non-wildcard) bytes at this anchor
    bytes: Vec<u8>,
}

impl Anchor {
    /// Creates a new anchor, making sure `bytes.len() != 0`
    fn new(offset: usize, bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() == 0 {
            None
        } else {
            Some(Self { offset, bytes })
        }
    }

    /// Return the score (unlikeliness of it appearing in memory) of the anchor
    fn score(&self) -> f64 {
        let len = self.bytes.len() as f64;

        // Count how often the bytes appear
        let mut counts = [0u16; 256];
        let mut unique = 0;

        for &b in self.bytes.iter() {
            if counts[b as usize] == 0 { unique += 1; }
            counts[b as usize ] = counts[b as usize].checked_add(1)
                .expect("That's an awfuly large pattern you're scanning there");
        }

        // Calculate how diverse this anchor is
        let diversity = unique as f64 / len;

        // Entropy estimation
        let entropy = {
            counts.iter().filter(|&&c| c > 0).map(|&c| {
                let p = c as f64 / len;
                -p * p.log2()
            }).sum::<f64>()
        };

        // Penalty if all bytes are the same
        let all_same_penalty = if unique == 1 { 5.0 } else { 0.0 };

        // Slight penalty if mostly the same byte (e.g., 01 01 01 02)
        let max_count = {
            let mut counts = [0u32; 256];
            for &b in self.bytes.iter() {
                counts[b as usize] += 1;
            }
            *counts.iter().max().unwrap_or(&0) as f64 / len
        };
        let repetition_penalty = if max_count > 0.9 { 3.0 } else { 0.0 };

        // Calculate the base score
        let base_score = len * 1.5
            + entropy * 2.0
            + diversity * 3.0;

        // Return the actual score including penalties
        base_score - all_same_penalty - repetition_penalty
    }
}

/// Anchors parsed from the command, sorted by their heuristic score
#[derive(Debug)]
struct Pattern {
    /// Vector of the actual anchors
    anchors: Vec<Anchor>,
}

impl Pattern {
    /// Extract the anchors of a pattern string, sorted by their heuristic score
    /// (i.e. their chance of _not_ appearing in memory)
    fn parse_scored_anchors(args: Option<&[&str]>) -> Result<Self, String> {
        // Make sure we have stuff to work with
        let parts = args.ok_or("No pattern provided".to_string())?;

        // Accumulates final anchors
        let mut anchors = Vec::new();

        // Temporarily holds contiguous known bytes
        let mut current_bytes = Vec::new();

        // Start index of the current anchor in `parts`
        let mut anchor_start = 0;

        // Parse the anchors
        for (i, part) in parts.iter().enumerate() {
            if *part == "??" {
                // Wildcard encountered: close current anchor (if any)
                let anchor = Anchor::new(
                    anchor_start, std::mem::take(&mut current_bytes));
                if let Some(anchor) = anchor {
                    anchors.push(anchor);
                }

                // Move on to the next part
                continue;
            }

            // Try to parse the string as a hexadecimal byte
            match u8::from_str_radix(part, 16) {
                Ok(byte) => {
                    // If this is the first bye in a new anchor, record the
                    // start index
                    if current_bytes.is_empty() {
                        anchor_start = i;
                    }
                    current_bytes.push(byte);
                }
                Err(_) => return Err(format!("Invalid byte '{}'", part)),
            }
        }

        // If we have any bytes left, push the final anchor
        if let Some(anchor) = Anchor::new(anchor_start, current_bytes) {
            anchors.push(anchor);
        }

        // If there are no anchors, we were given an all-wildcard pattern
        if anchors.is_empty() {
            return Err("Won't search for an all-wildcard pattern.".to_string());
        }

        // Sort the anchors
        let mut anchors = Self {
            anchors,
        };
        anchors.sort_by_score();

        Ok(anchors)
    }

    /// Get a reference to the anchor with the highest score
    fn get_best(&self) -> &Anchor {
        // Because this struct is automatically sorted when created, the first
        // anchor is the best one
        &self.anchors[0]
    }

    /// An iterator over occurrences of the pattern, specifically the offsets
    /// into `mem` where the pattern appears
    fn find_pattern_iter<'a>(&'a self, mem: &'a [u8]) -> PatternIterator<'a> {
        PatternIterator::new(self, mem)
    }

    /// Sort the anchors by their heuristic score
    fn sort_by_score(&mut self) {
        self.anchors.sort_unstable_by(|a, b|
            a.score().partial_cmp(&b.score()).unwrap());
    }
}

/// Iterator over found occurrences of a pattern in memory
struct PatternIterator<'a> {
    /// The anchors of the pattern we're scanning
    pattern: &'a Pattern,

    /// The memory we're scanning for the pattern
    mem: &'a [u8],

    /// Occurrences of the best anchor in `mem`
    occurrences: memmem::FindIter<'a, 'a>,
}

impl<'a> PatternIterator<'a> {
    fn new(pattern: &'a Pattern, mem: &'a [u8]) -> Self {
        // Build the iterator over the memory
        let occurrences = memmem::find_iter(mem, &pattern.get_best().bytes);

        Self { pattern, mem, occurrences }
    }
}

impl<'a> Iterator for PatternIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next occurrence
        'iter: while let Some(occurred_idx) = self.occurrences.next() {
            let occurred_idx: isize = occurred_idx.try_into().unwrap();

            // Get the best anchor's offset. We'll use this to calculate the
            // offsets of other anchors
            let best_offset = self.pattern.get_best().offset as isize;

            // Validate the anchors
            for anchor in self.pattern.anchors.iter().skip(1) {
                // Get the offset from the anchor used in the initial scan
                let offset = anchor.offset as isize - best_offset;

                // Get the bytes we're matching against this anchor
                let start = usize::try_from(occurred_idx + offset).unwrap();
                let end   = start + anchor.bytes.len();

                // skip if this anchor would go out of bounds
                if end > self.mem.len() {
                    continue 'iter;
                }

                // If any of the anchors don't match, these bytes don't follow
                // the pattern
                if &self.mem[start..end] != anchor.bytes {
                    continue 'iter;
                }
            }

            // All anchors valid, return the offset into memory where the
            // pattern appears
            return Some(usize::try_from(occurred_idx - best_offset).unwrap())
        }
        None
    }
}
