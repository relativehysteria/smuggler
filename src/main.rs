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

    // // Create the CLI and run the application! yay
    // let mut cli = Cli::new(pid, ">> ".to_string())?;
    let mut regions = smug::proc_maps::Maps::rw_regions(pid).unwrap();
    smug::read_remote::populate_regions(pid, regions.0.as_mut_slice());
    for region in &regions.0 {
        println!("{:X?} {:x?} {:?}", region.memory.is_some(), region.addr().start, region.path());
    }

    Ok(())
}
