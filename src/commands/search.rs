use crate::commands::{parse_arg, parse_value};

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

fn handler(_: &mut crate::Scanner, _: &[&str]) -> crate::commands::Result {
    // Parse the value type from the first argument
    let value = parse_value(args.get(0))?;

    // Parse the start and end addresses
    let start = parse_arg::<u64>(args.get(1), "Start address")?;
    let end   = parse_arg::<u64>(args.get(2), "End address")?;

    // If end is undefined, default to the maximum address
    let end = if end == 0 { u64::MAX } else { end };
}
