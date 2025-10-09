crate::register_command_handler!(
    handler, ["m", "maps"],
    "Prints out scannable address maps",
    "Takes no arguments."
);

fn handler(scanner: &mut crate::Scanner, _: &[String]) -> String {
    crate::Maps::rw_regions(scanner.pid())
        .map(|maps| {
            maps.0.into_iter()
            .map(|region| format!("{}", region))
            .intersperse("\n".to_string())
            .collect()
        })
    .unwrap_or_else(|e| format!("Couldn't parse maps: {:?}", e))
}
