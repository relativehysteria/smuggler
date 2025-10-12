use crate::commands::{parse_arg, get_addr_region};

crate::register_command_handler!(
    handler, ["r", "reg", "region"],
    "Show what region an address is mapped in.",
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

    // Print the region that maps this address
    if let Some(region) = get_addr_region(&maps.0, addr) {
        println!("{region}");
    }

    Ok(())
}
