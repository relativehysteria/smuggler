use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["r", "reg", "region"],
    "Show what region an address is mapped in",
r#"`<address>`
* `address` - The address whose region will be shown
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the number of addresses to show
    let addr = parse_arg::<u64>(args.get(1), "Address")?;

    // Get the regions
    let maps = crate::Maps::all_regions(s.pid())
        .map_err(|_| "Couldn't read regions".to_string())?;

    // Binsearch would be faster but this will be instant anyways
    for region in maps.0.iter() {
        // Don't overiterate. Stop when we're too far
        if region.addr.start > addr { break; }

        if region.addr.start <= addr && region.addr.end > addr {
            println!("{region:X?}");
            break;
        }
    }

    Ok(())
}
