//! Exits the application

crate::register_command_handler!(["exit", "quit"], handler);

fn handler(_: &mut crate::Scanner, _: &[String]) -> String {
    std::process::exit(0);
}
