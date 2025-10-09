crate::register_command_handler!(
    handler, ["search", "scan"],
    "Searches memory for this and that",
    "Takes the value to search and the address start and end"
);

fn handler(_: &mut crate::Scanner, _: &[String]) -> crate::commands::Result {
    Ok("Scanning!:)".to_string());
}
