//! Command line interface

use rustyline::{Editor, history::FileHistory, config::Config};
use crate::{Pid, Error, Scanner};
use crate::commands::{CommandHandler, get_command_handlers};

/// Handler for the interface
#[derive(Debug)]
pub struct Cli {
    /// The inner readline interface
    rl: Editor<(), FileHistory>,

    /// The file where history will be saved
    history_file: String,

    /// The prompt that appears on the command line
    prompt: String,

    /// The internal scanner state
    scanner: Scanner,

    /// A mapping of string commands to their handlers
    commands: std::collections::HashMap<String, CommandHandler>,
}

impl Cli {
    /// Create a new scanner interface for the following PID
    pub fn new(pid: Pid, prompt: String) -> crate::Result<Self> {
        // Make sure we can read from this process
        let _ = crate::Maps::accessible(pid)?;

        // Save the history file path
        let history_file = format!("/tmp/smug_{}", pid.0);

        // Create the editor
        let config = Config::builder()
            .max_history_size(0xFFFF).map_err(Error::Cli)?
            .auto_add_history(true)
            .tab_stop(4)
            .indent_size(4)
            .build();

        let mut rl = Editor::with_config(config).map_err(Error::Cli)?;

        // Load history if possible
        let _ = rl.load_history(history_file.as_str());

        // Initialize the command handler registry
        let commands = get_command_handlers();
        // println!("{commands:?}");

        // Create the scanner
        let scanner = Scanner::new(pid);

        Ok(Self { rl, history_file, prompt, commands, scanner })
    }

    /// Get the next command, saving it to the history file if valid
    pub fn next_command(&mut self) -> crate::Result<String> {
        let cmd = self.rl.readline(self.prompt.as_str()).map_err(Error::Cli)?;
        let _ = self.rl.save_history(self.history_file.as_str());
        Ok(cmd)
    }

    /// The main loop of the application!
    pub fn main_loop(&mut self) -> crate::Result<()> {
        loop {
            // Get the next command
            let line = self.next_command()?;

            // Split it into substrings
            let cmd: Vec<&str> = line
                .split_whitespace()
                .collect();

            // If no command was given, go next
            if cmd.len() == 0 { continue; }

            // Try to get a handler for this command
            if let Some(handler) = self.commands.get(cmd[0]) {
                // Save command to history if execution is successful
                match handler(&mut self.scanner, &cmd) {
                    Ok(_) => (),
                    Err(e)  => println!("!!! {e}"),
                }
            } else {
                println!("Unknown command!");
            }
        }
    }
}
