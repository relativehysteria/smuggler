use crate::commands::{parse_arg, parse_value, parse_constraints};

crate::register_command_handler!(
    handler, ["sb", "sw", "sd", "sq", "sB", "sW", "sD", "sQ", "sf", "sF"],
    "Scan the memory for values.",
r#"`<start_address> <end_address> <constraints>`
* `start_address` - Start searching from this address. If this is `0`, the
   search will start from the first readable memory region.
* `end_address` - Stop searching at this address. If this is `0`, the search
   will stop at the last readable memory region.
* `constraints` - The constraints by which to search
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the value type from the first argument
    let mut value = parse_value(args.get(0))?;

    // Parse the start and end addresses
    let start = parse_arg::<u64>(args.get(1), "Start address")?;
    let end   = parse_arg::<u64>(args.get(2), "End address")?;

    // If end is undefined, default to the maximum address
    let end = if end == 0 { u64::MAX } else { end };

    // Parse the constraints
    let constraints = parse_constraints(&args[3..], value)?;

    // Get the memory map
    let maps = crate::proc_maps::Maps::rw_regions(s.pid())
        .map_err(|e| format!("Couldn't parse memory map: {:?}", e))?;

    // Get the iovec batches
    let iovecs = maps.chunks(core::ops::Range { start, end });

    // Search for the values and save off the adresses where they're found
    let mut matches = Vec::new();

    for batch in iovecs.into_iter() {
        // Read the memory
        let memory = crate::remote::read_vecs(s.pid(), &batch);

        // Retain only those chunks of memory that have been successfully read
        let chunks = batch.iter().zip(memory.into_iter())
            .filter(|(_, mem)| mem.is_some())
            .map(|(iovec, mem)| (iovec, mem.unwrap()));

        // Go through each region and scan for the value
        for (iovec, mem) in chunks {
            // Go through the region in chunks
            for (offset, chunk) in mem.chunks_exact(value.bytes()).enumerate() {
                // Update the value
                value.from_le_bytes(chunk);

                // Check that constraints match and if they do, save the address
                if constraints.iter().all(|x| x.check(value)) {
                    let abs = iovec.base + offset as u64 * value.bytes() as u64;
                    matches.push(abs);
                }
            }
        }
    }

    // Print the results
    if matches.is_empty() {
        println!("No matches for in {:#x?}..{:#x?}", start, end);
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
