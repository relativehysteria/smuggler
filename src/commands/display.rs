use core::num::NonZero;
use crate::remote;
use crate::commands::parse_arg;

crate::register_command_handler!(
    handler, ["db", "dw", "dd", "dq", "dB", "dW", "dD", "dQ", "df", "dF"],
    "Display memory bytes.",
r#"`<address> [<length>]`
* `address` -- The address where the display should start in hex
* `length` -- The amount of bytes to show in hex. `0x40` by default.
"#
);

/// Number of byte values preinted per line
const VALUES_PER_LINE: usize = 16;

fn handler(s: &mut crate::Scanner, args: &[&str]) -> crate::commands::Result {
    // Parse the value type from the first argument
    let value = args
        .get(0)
        .and_then(|arg| arg.chars().nth(1))
        .map(crate::num::Value::default_from_letter)
        .ok_or("Missing or invalid type specifier")?;

    // Parse the address
    let addr = parse_arg::<u64>(args.get(1), "Start address")?;

    // Parse the length
    let len = args.get(2)
        .map(|s| crate::num::parse::<usize>(s)
            .map_err(|e| format!("Length not a valid number: {:?}", e)))
        .transpose()?
        .unwrap_or(0x40);

    // Make sure we have nonzero length
    let len = NonZero::new(len).ok_or("Length must not be zero!")?;

    // Read the memory
    let mem = remote::read(s.pid(), addr, len)
        .ok_or(format!("Couldn't read remote memory at 0x{:X?}", addr))?;

    // Print the newline header
    print!("\x1b[0;34m{:016x}\x1b[0m: ", addr);

    let bytes_per_value = value.bytes();
    let vals_per_line = VALUES_PER_LINE / bytes_per_value;

    for (i, chunk) in mem.chunks(bytes_per_value).enumerate() {
        // Print value or placeholder
        if chunk.len() == bytes_per_value {
            let mut val = value;
            val.from_le_bytes(chunk);

            // Check whether the value is a valid readable pointer
            let len = NonZero::new(1).unwrap();
            let is_valid = remote::read(s.pid(), val.as_u64(), len).is_some();

            if is_valid {
                print!("\x1b[0;32m{val}\x1b[0m ");
            } else {
                print!("{val} ");
            }
        } else {
            // Incomplete chunk -- print placeholder
            print!("{:?} ", "?".repeat(value.display()));
        }

        // Newline + ASCII dump every 16 bytes
        if (i + 1) % vals_per_line == 0 {
            let base = i / vals_per_line * 16;
            let ascii_slice = &mem[base..mem.len().min(base + 16)];

            for &b in ascii_slice {
                let c = if b.is_ascii_graphic() { b as char } else { '.' };
                print!("{c}");
            }
            println!();

            // Stop if all values have been printed
            if i + 1 == mem.len() / bytes_per_value {
                break;
            }

            // Print next line header.
            let next_addr = addr + ((i + 1) * bytes_per_value) as u64;
            print!("\x1b[34m{next_addr:016x}\x1b[0m: ");
        }
    }

    Ok(())
}
