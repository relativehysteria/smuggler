crate::register_command_handler!(
    handler, ["search", "scan"],
    "Searches memory for this and that",
    "Takes the value to search and the address start and end"
);

fn handler(_scanner: &mut crate::Scanner, _args: &[String]) -> String {
    println!("Scanning!:)");
}
