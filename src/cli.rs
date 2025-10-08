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
        let scanner = Scanner::new();

        Ok(Self { rl, history_file, prompt, commands, scanner })
    }

    /// Get the next command, saving it to the history file if valid
    pub fn next_command(&mut self) -> crate::Result<String> {
        // Get the command
        let cmd = self.rl.readline(self.prompt.as_str())
            .map_err(Error::Cli);

        // Save if valid
        if cmd.is_ok() {
            let _ = self.rl.save_history(self.history_file.as_str());
        }

        cmd
    }

    /// The main loop of the application!
    pub fn main_loop(&mut self) -> crate::Result<()> {
        // TODO: only save actual successful commands, not everything
        loop {
            // Get the next command and its arguments
            let cmd_line = self.next_command()?;
            let cmd_line: Vec<String> = cmd_line
                .split_whitespace()
                .map(|word| word.to_lowercase())
                .collect();

            // If no command was given, go next
            if cmd_line.len() == 0 { continue; }

            // Attempt to get a handler for this command
            match self.commands.get(&cmd_line[0]) {
                None => println!("Unknown command"),
                Some(handler) => {
                    let ret = handler(&mut self.scanner, &cmd_line[1..]);
                    println!("{ret}");
                }
            }
        }
    }
}
