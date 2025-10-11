use memchr::memmem;
use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["ss"],
    "Search for a string.",
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

    // Get the memory map
    let maps = crate::proc_maps::Maps::rw_regions(s.pid())
        .map_err(|e| format!("Couldn't parse memory map: {:?}", e))?;

    // Get the iovec batches
    let iovecs = maps.chunks(core::ops::Range { start, end });

    // Search for the string and save off the adresses where it's found
    let mut matches = Vec::new();
    let needle = string.as_bytes();

    for batch in iovecs.into_iter() {
        // Read the memory
        let memory = crate::remote::read_vecs(s.pid(), &batch);

        // Retain only those chunks of memory that have been successfully read
        let chunks = batch.iter().zip(memory.into_iter())
            .filter(|(_, mem)| mem.is_some())
            .map(|(iovec, mem)| (iovec, mem.unwrap()));

        // Go through each region and scan for the string
        for (iovec, mem) in chunks {
            for offset in memmem::find_iter(&mem, needle) {
                let absolute = iovec.base + offset as u64;
                matches.push(absolute);
            }
        }
    }

    // Print the results
    if matches.is_empty() {
        println!("No matches.");
    } else {
        if matches.len() > 10 {
            println!("Found {} matches.", matches.len());
        } else if matches.len() == 1 {
            println!("Found 1 match at:");
            println!("  0x{:X}", matches[0])
        } else {
            println!("Found {:?} matches at:", matches.len());
            for addr in matches.iter() {
                println!("  0x{:X}", addr);
            }
        }

        // Save the matches
        s.results = matches;
    }

    Ok(())
}
