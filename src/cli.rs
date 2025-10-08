//! Command line interface

use rustyline::{Editor, history::FileHistory, config::Config};
use crate::{Pid, Error};

/// Handler for the interface
#[derive(Debug)]
pub struct Cli {
    /// The inner readline interface
    rl: Editor<(), FileHistory>,

    /// The file where history will be saved
    history_file: String,

    /// The prompt for each command
    prompt: String,
}

impl Cli {
    /// Create a new scanner interface for the following PID
    pub fn new(pid: Pid, prompt: String) -> crate::Result<Self> {
        // Save the history file path
        let history_file = format!("/tmp/smug_{}", pid.0);

        // Create the editor
        let config = Config::builder()
            .max_history_size(0xFFFF).map_err(Error::CliError)?
            .auto_add_history(true)
            .tab_stop(4)
            .indent_size(4)
            .build();

        let mut rl = Editor::with_config(config).map_err(Error::CliError)?;

        // Load history if possible
        let _ = rl.load_history(history_file.as_str());

        Ok(Self { rl, history_file, prompt })
    }

    /// Get the next command, saving it to the history file if valid
    pub fn next_command(&mut self) -> crate::Result<String> {
        // Get the command
        let cmd = self.rl.readline(self.prompt.as_str())
            .map_err(Error::CliError);

        // Save if valid
        if cmd.is_ok() {
            let _ = self.rl.save_history(self.history_file.as_str());
        }

        cmd
    }
}
