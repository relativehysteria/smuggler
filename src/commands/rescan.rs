use crate::commands::{parse_value, parse_constraints, scan_batch};
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
    let value = parse_value(args.get(0))?;

    // Parse the constraints
    let constraints = parse_constraints(&args[1..], value)?;

    // Create iovecs for the addresses returned by the previous scan
    let bytes = core::num::NonZero::new(value.bytes()).unwrap();
    let iovecs: Vec<IoVec> = s.results.iter()
        .map(|&addr| IoVec::new(addr, bytes))
        .collect();

    // Search for the values and save off the adresses where they're found
    let mut matches = Vec::new();

    for batch in iovecs.chunks(CHUNK_SIZE / value.bytes()) {
        scan_batch(s.pid(), &mut matches, batch, value, &constraints);
    }

    crate::commands::print_and_save_results(s, matches);

    Ok(())
}
