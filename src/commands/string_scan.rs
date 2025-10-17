use memchr::memmem;
use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["ss", "ss16", "ss32"],
    "Search for a string (or a UTF-16/UTF-32 LE wide string)",
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
    let needle = match args[0] {
        c if c.ends_with("16") => string
            .encode_utf16()
            .flat_map(|unit| unit.to_le_bytes())
            .collect(),

        c if c.ends_with("32") => string
            .chars()
            .flat_map(|ch| (ch as u32).to_le_bytes())
            .collect(),

        _ => string.as_bytes().to_vec(),
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
