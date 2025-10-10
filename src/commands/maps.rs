crate::register_command_handler!(
    handler, ["m", "maps"],
    "Prints out scannable address maps",
    "Takes no arguments."
);

fn handler(s: &mut crate::Scanner, _: &[&str]) -> crate::commands::Result {
    let maps: String = crate::Maps::rw_regions(s.pid())
        .map(|maps| {
            maps.0.into_iter()
            .map(|region| format!("{}", region))
            .intersperse("\n".to_string())
            .collect()
        })
    .map_err(|e| format!("Couldn't parse maps: {:?}", e))?;
    println!("{maps}");
    Ok(())
}
