//! Exits the application

crate::register_command_handler!(
    handler, ["exit", "quit"],
    "Exits the application immediately.",
    "Takes no arguments."
);

fn handler(_: &mut crate::Scanner, _: &[String]) -> String {
    std::process::exit(0);
}
