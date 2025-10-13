use core::num::NonZero;
use crate::remote;
use crate::commands::{parse_arg, parse_value};

crate::register_command_handler!(
    handler, ["db", "dw", "dd", "dq", "dB", "dW", "dD", "dQ", "df", "dF"],
    "Display memory bytes.",
r#"`<address> [<length>]`
* `address` -- The address where the display should start in hex
* `length` -- The amount of bytes to show in hex. [`DEFAULT_BYTES`] by default.
"#
);

/// Number of byte values preinted per line
const VALUES_PER_LINE: usize = 16;

/// Default amount of of bytes to show
pub const DEFAULT_BYTES: usize = 0x40;

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the value type from the first argument
    let value = parse_value(args.get(0))?;

    // Parse the address
    let addr = parse_arg::<u64>(args.get(1), "Start address")?;

    // Parse the length
    let len = args.get(2)
        .map(|s| crate::num::parse::<usize>(s)
            .map_err(|e| format!("Length not a valid number: {:?}", e)))
        .transpose()?
        .unwrap_or(DEFAULT_BYTES);

    // Make sure we have nonzero length
    let len = NonZero::new(len).ok_or("Length must not be zero!")?;

    // Read the memory
    let mem = remote::read(s.pid(), addr, len)
        .ok_or(format!("Couldn't read remote memory at 0x{:X?}", addr))?;

    // Print the newline header
    print!("\x1b[0;34m{:016x}\x1b[0m │ ", addr);

    // Derived constants:
    // * bytes_per_value: how many bytes make up one displayed unit
    // * vals_per_line:   how many values fit in one 16-byte text line
    // * total_values:    number of value-sized chunks, rounded up
    let bytes_per_value = value.bytes();
    let vals_per_line = VALUES_PER_LINE / bytes_per_value;
    let total_values = (mem.len() + bytes_per_value - 1) / bytes_per_value;

    // Iterate over memory one value at a time
    for (i, chunk) in mem.chunks(bytes_per_value).enumerate() {
        if chunk.len() == bytes_per_value {
            // Full chunk: convert the bytes into the requested value type
            let mut val = value;
            val.from_le_bytes(chunk);

            // Check whether the value is a valid readable pointer. If it is, we
            // colorize it
            let len = NonZero::new(1).unwrap();
            let is_valid = remote::read(s.pid(), val.as_u64(), len).is_some();

            if is_valid {
                print!("\x1b[0;32m{val}\x1b[0m ");
            } else {
                print!("{val} ");
            }
        } else {
            // Partial chunk: occurs when the requested memory size is not an
            // exact multiple of the value size. We can't safely decode it, so
            // we print a visual placeholder instead.
            print!("{} ", "?".repeat(value.display()));
        }

        // When we've printed a full line of values, emit the ASCII dump.
        if (i + 1) % vals_per_line == 0 {
            // Compute which bytes correspond to this visual line
            let base = (i + 1 - vals_per_line) * bytes_per_value;
            let ascii_slice = &mem[base..mem.len().min(base + VALUES_PER_LINE)];

            // ASCII view
            print!("│ ");
            for &b in ascii_slice {
                let c = if b.is_ascii_graphic() { b as char } else { '.' };
                print!("{c}");
            }
            println!();

            // If more values remain, print the next address header
            if i + 1 != total_values {
                let next_addr = addr + ((i + 1) * bytes_per_value) as u64;
                print!("\x1b[34m{next_addr:016x}\x1b[0m │ ");
            }
        }
    }

    // Handle final line if oncomplete
    if total_values % vals_per_line != 0 {
        // Compute where the remaining bytes begin
        let remaining_start = (total_values / vals_per_line) * VALUES_PER_LINE;
        let ascii_slice = &mem[remaining_start..];
        let pad_len = VALUES_PER_LINE - ascii_slice.len();

        // Pad spacing to match value columns visually
        let width = pad_len / bytes_per_value * (value.display() + 1);
        print!("{:width$}", "");

        // Print the trailing ASCII
        print!("│ ");
        for &b in ascii_slice {
            let c = if b.is_ascii_graphic() { b as char } else { '.' };
            print!("{c}");
        }
        println!();
    }

    Ok(())
}
