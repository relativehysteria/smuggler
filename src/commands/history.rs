use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["h", "hist", "history"],
    "Show the addresses of the last scan.",
r#"`<n_addresses>`
* `n_addresses` - Maximum number of addresses to show. Starts at the first one.
  If this is `0`, all addresses will be shown
"#
);

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the number of addresses to show
    let n_show = parse_arg::<usize>(args.get(1), "Number of entries to show")?;
    let n_show = if n_show == 0 { usize::MAX } else { n_show };

    // Show the addresses
    for entry in s.results.iter().take(n_show) {
        println!("0x{entry:X}");
    }

    Ok(())
}
