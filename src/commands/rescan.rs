use crate::commands::{parse_value, parse_arg, parse_constraints, scan_batch};
use crate::{CHUNK_SIZE, remote::IoVec};

crate::register_command_handler!(
    handler, ["ub", "uw", "ud", "uq", "uB", "uW", "uD", "uQ", "uf", "uF"],
    "Rescan the results from a previous scan for new values.",
r#"`<id> <constraints>`
* `id` - The index of the history entry which will be rescanned. If this is `0`,
  the last history entry will be used.
* `constraints` - The constraints by which to scan
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the value type and the constraints
    let value = parse_value(args.get(0))?;

    // Parse the history entry whose results will be used for this scan.
    let idx = parse_arg::<usize>(args.get(1), "History index")?;

    // Normalize the index to 0-based, using the last index when the input is 0
    let idx = idx.checked_sub(1).unwrap_or(s.results.len().saturating_sub(1));

    // Parse the constraints
    let constraints = parse_constraints(&args[2..], value)?;

    // Create iovecs for the addresses returned by the previous scan
    let bytes = core::num::NonZero::new(value.bytes()).unwrap();
    let iovecs: Vec<IoVec> = if let Some(results) = s.results.get(idx) {
        results.iter()
            .map(|&addr| IoVec::new(addr, bytes))
            .collect()
    } else {
        return Ok(());
    };

    // Search for the values and save off the adresses where they're found
    let mut matches = Vec::new();

    for batch in iovecs.chunks(CHUNK_SIZE / value.bytes()) {
        scan_batch(s.pid(), &mut matches, batch, value, &constraints);
    }

    crate::commands::print_and_save_results(s, matches);

    Ok(())
}
