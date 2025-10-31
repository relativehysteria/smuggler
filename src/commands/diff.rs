use std::cmp::Ordering;

crate::register_command_handler!(
    handler, ["d", "diff"],
    "List addresses from the last scan not present in the scan before it",
    "Takes no arguments."
);


fn handler(s: &mut crate::Scanner, _args: &[&str]) -> crate::commands::Result {
    // If we haven't had 2 scans yet, quit early
    if s.results.len() != 2 { return Ok(()); }

    // Get the scans
    let base = s.results.front().unwrap();
    let last = s.results.back().unwrap();

    // Merge compare the results
    let mut i = 0;
    let mut j = 0;
    let mut diff = Vec::new();

    while i < last.len() && j < base.len() {
        match last[i].cmp(&base[j]) {
            Ordering::Less    => { i += 1; diff.push(last[i - 1]); },
            Ordering::Equal   => { i += 1; j += 1; }
            Ordering::Greater => {         j += 1; }
        }
    }

    // Any remaining elements in `last` are unique
    diff.extend_from_slice(&last[i..]);

    crate::commands::print_results(s.pid(), &diff, usize::MAX);

    Ok(())
}
