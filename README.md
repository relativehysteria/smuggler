# smuggler

rewrite of cheat engine's memory scanning for linux.

this is an effort to get similar functionality as cheat engine into linux
because pince sucks and other tools don't fit my use case perfectly :)

## commands

all implemented commands and their documentation can be found within their
source files in the [commands](src/commands) directory.

the `register_command_handler!()` macro requires that each command is
documented; `cargo doc` will therefore create the most up-to-date documentation.
