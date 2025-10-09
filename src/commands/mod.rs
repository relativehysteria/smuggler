//! Commands handleable by this application
//!
//! This module implements a plugin-like architecture for handling commands,
//! using linker sections to automatically register command handlers declared
//! throughout the codebase.
//!
//! Command handlers are declared with the [`crate::register_command_handler`]
//! macro. The macro places the handler in a special linker section, which is
//! walked at runtime to construct a full list of available commands.

use std::collections::HashMap;
use crate::Scanner;


// COMMAND REGISTRATION ────────────────────────────────────────────────────────
// Things are imported using this macro to automatically expose command
// documentation in `cargo doc`

macro_rules! import_command {
    ($name:ident) => {
        mod $name;
        pub use $name::HANDLER as $name;
    };
}

import_command!(exit);
import_command!(maps);

// ─────────────────────────────────────────────────────────────────────────────

/// Command handler type
///
/// A command will be given the scanner state and a list of arguments that it
/// can then handle.
pub type CommandHandler = fn(&mut Scanner, &[String]) -> String;

/// A single command handler registration entry
///
/// This is a pair of:
/// - A static list of command names (`&[&str]`) that this handler responds to
/// - A function pointer to the handler itself (`CommandHandler`)
///
/// For example, a handler might register for multiple aliases like `["foo",
/// "bar"]`.
type HandlerMapping = (&'static [&'static str], CommandHandler);

/// Macro to register a new command handler
///
/// This macro takes a list of command strings and a function name,
/// and places a reference to the handler in a custom linker section.
///
/// # Example
/// ```rust
/// register_command_handler!(["help", "h"], handle_help_command);
/// ```
///
/// ## How it works:
/// - It creates a static array of command strings.
/// - It creates a `HandlerMapping` pairing the command array and the function.
/// - It then places a **reference** to that mapping in a special section of the
///   binary (`.command_handlers`) using `#[link_section]`.
///
/// These references are gathered at runtime using start/end symbols
/// emitted by the linker.
#[macro_export]
macro_rules! register_command_handler {
    ($func:ident, [$($cmd:literal),+], $desc:literal, $args_doc:literal) => {
        #[doc = concat!(
            "Command handler for: ",
            $( "`", $cmd, "`", ", " ),+,
            "\n## Description\n",
            $desc,
            "\n## Arguments\n",
            $args_doc
        )]
        #[used]
        #[unsafe(link_section = ".command_handlers")]
        pub static HANDLER: &$crate::commands::HandlerMapping = {
            static CMD: &[&str] = &[$($cmd),+];
            static HANDLER: $crate::commands::HandlerMapping = (CMD, $func);
            &HANDLER
        };
    };
}


unsafe extern "Rust" {
    static __start_command_handlers: *const &'static HandlerMapping;
    static __end_command_handlers: *const &'static HandlerMapping;
}

/// Returns all registered command handlers from the linker section
///
/// This function constructs a slice from the `.command_handlers` section
/// by taking the start and end symbols and calculating the number of entries.
fn get_raw_command_handlers() -> &'static [&'static HandlerMapping] {
    unsafe {
        let start = &__start_command_handlers as *const *const &HandlerMapping;
        let end = &__end_command_handlers as *const *const &HandlerMapping;
        let count = end.offset_from(start) as usize;
        core::slice::from_raw_parts(start as *const &HandlerMapping, count)
    }
}

/// Build a map of all command strings to their respective handler functions
///
/// This function iterates over all registered handlers and builds
/// a `HashMap` mapping command names to their handlers.
///
/// If multiple handlers register the same command string, the last one wins.
pub fn get_command_handlers() -> HashMap<String, CommandHandler> {
    let mut map = HashMap::new();
    for handler in get_raw_command_handlers() {
        for &cmd in handler.0.iter() {
            map.insert(cmd.to_string(), handler.1);
        }
    }
    map
}
