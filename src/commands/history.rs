use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["h", "hist", "history"],
    "Show the addresses of a previous scan.",
r#"`<id> <n_addresses>`
* `id` - The index of the history entry whose results will be shown. If this is
  `0`, the last history entry will be used.
* `n_addresses` - Maximum number of addresses to show. Starts at the first one.
  If this is `0`, all addresses will be shown
"#
);


fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the index whose results will be used for this scan and normalize it
    // to be 0-based, using the last index when the input is 0
    let idx = parse_arg::<usize>(args.get(1), "History index")?;
    let idx = idx.checked_sub(1).unwrap_or(s.results.len().saturating_sub(1));

    // Parse the number of addresses to show
    let n_show = parse_arg::<usize>(args.get(2), "Number of entries to show")?;
    let n_show = if n_show == 0 { usize::MAX } else { n_show };

    // Show the addresses
    if let Some(results) = s.results.get(idx) {
        crate::commands::print_results(s.pid(), results, n_show);
    }

    Ok(())
}
