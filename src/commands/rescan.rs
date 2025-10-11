use crate::commands::{parse_value, parse_constraints};
use crate::{CHUNK_SIZE, remote::IoVec};

crate::register_command_handler!(
    handler, ["ub", "uw", "ud", "uq", "uB", "uW", "uD", "uQ", "uf", "uF"],
    "Rescan the resutls from previous scan for new values.",
r#"`<constraints>`
* `constraints` - The constraints by which to scan
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the value type and the constraints
    let mut value = parse_value(args.get(0))?;

    // Parse the constraints
    let constraints = parse_constraints(&args[1..], value)?;

    // Create iovecs for these addresses
    let bytes = core::num::NonZero::new(value.bytes()).unwrap();
    let iovecs: Vec<IoVec> = s.results.iter()
        .map(|&addr| IoVec::new(addr, bytes))
        .collect();

    // Search for the values and save off the adresses where they're found
    let mut matches = Vec::new();

    for batch in iovecs.chunks(CHUNK_SIZE / value.bytes()) {
        // Read the memory
        let memory = crate::remote::read_vecs(s.pid(), batch);

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
