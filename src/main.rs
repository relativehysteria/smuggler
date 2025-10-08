use rustyline::Editor;
use rustyline::history::FileHistory;
use smug::{Pid, cli::Cli};

fn main() -> smug::Result<()> {
    // Get the arguments
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        println!("Usage: {} <pid>", args.get(0).unwrap());
        return Ok(());
    }

    // Get the requested pid
    let pid = Pid::try_from(args[1].as_str())?;

    // Create the CLI and run the application! yay
    let mut cli = Cli::new(pid, ">> ".to_string())?;

    Ok(())
}
