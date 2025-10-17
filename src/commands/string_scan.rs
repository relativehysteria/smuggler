use memchr::memmem;
use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["ss", "ss16", "ss32"],
    "Search for a string (or a UTF-16 or UTF-32 wide string)",
r#"`<start_address> <end_address> <string>`
* `start_address` - Start searching from this address. If this is `0`, the
   search will start from the first readable memory region.
* `end_address` - Stop searching at this address. If this is `0`, the search
   will stop at the last readable memory region.
* `string` - The string to search for.
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the start and end addresses
    let start = parse_arg::<u64>(args.get(1), "Start address")?;
    let end   = parse_arg::<u64>(args.get(2), "End address")?;

    // If end is undefined, default to the maximum address
    let end = if end == 0 { u64::MAX } else { end };

    // Get the string
    let string = args.get(3..)
        .filter(|parts| !parts.is_empty())
        .map(|parts| parts.join(" "))
        .ok_or("String missing!")?;

    // Encode the string depending on the command we're handling
    let cmd = args[0];
    let needle = if cmd.ends_with("16") {
        let mut buf = Vec::with_capacity(string.len() * 2);
        for unit in string.encode_utf16() {
            buf.push((unit & 0xFF) as u8);
            buf.push((unit >> 8) as u8);
        }
        buf
    } else if cmd.ends_with("32") {
        let mut buf = Vec::with_capacity(string.len() * 4);
        for ch in string.chars() {
            let val = ch as u32;
            buf.push((val & 0xFF) as u8);
            buf.push(((val >>  8) & 0xFF) as u8);
            buf.push(((val >> 16) & 0xFF) as u8);
            buf.push(((val >> 24) & 0xFF) as u8);
        }
        buf
    } else {
        string.as_bytes().to_vec()
    };

    // Get the memory map
    let maps = crate::proc_maps::Maps::interesting_regions(s.pid())
        .map_err(|e| format!("Couldn't parse memory map: {:?}", e))?;

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

        // Go through each region and scan for the string
        for (iovec, mem) in chunks {
            for offset in memmem::find_iter(&mem, &needle) {
                let absolute = iovec.base + offset as u64;
                matches.push(absolute);
            }
        }
    }

    crate::commands::print_and_save_results(s, matches);

    Ok(())
}
