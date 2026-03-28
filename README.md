# Smuggler

A modular Cheat Engine inspired CLI memory scanner.

Currently Smuggler only supports read operations (it is merely a scanner), but
the command system is modular and implementing new commands is as simple as
writing a handler and registering it with
[`register_command_handler!()`](src/commands/mod.rs#L71). :)

# Expressions

The expression code is directly taken from
[mempeek](https://github.com/gamozolabs/mempeek) and its description is here for
completeness:

There is support for basic (no parentheses) expressions of various bases as
well as basic operations (addition, subtraction, multiplication, division),
allowing you to use an expression like `0x13370000+0o100*4` in any argument to a
command which expects a constant value.

The default base is 16.

## Types

Types may be one of the following:
- `b` - `u8`
- `w` - `u16`
- `d` - `u32`
- `q` - `u64`
- `B` - `i8`
- `W` - `i16`
- `D` - `i32`
- `Q` - `i64`
- `f` - `f32`
- `F` - `f64`

## Constraints

Constraints may be any one of the following:

- `=[val]`  - Equal to `[val]`
- `![val]`  - Not equal to `[val]`
- `>[val]`  - Greater than `[val]`
- `>=[val]` - Greater than or equal to `[val]`
- `<[val]`  - Less than `[val]`
- `<=[val]` - Less than or equal to `[val]`

# Commands

Commands are registered with the
[`register_command_handler!()`](src/commands/mod.rs#L71) macro which requires
each handler to have a description and argument list. This macro generates the
documentation for each registered handler, thus the most up-to-date
docs can be simply generated with `cargo doc`. 

I recommend you consult the generated documentation as many arguments also have
default values.

## Commonly used commands

- `q`: Quit the program.
- `h <n_addresses>`: Show an amount of addresses from the last scan.
- `m`: Show all readable address mappings.
- `r <addr>`: Show what region of memory an address is mapped in.
- `s[bwdqBWDQfF] <start_addr> <end_addr> <constraints>`: Scan the memory for
  values.
- `ss/ss16/ss32 <start_addr> <end_addr> <string>`: Scan the memory for a string
  (or a UTF-16/UTF-32 LE wide string).
- `sp <start_addr> <end_addr> <pattern>`: Search for non-overlapping occurrences
  of an IDA byte pattern string
  (nibble wildcard, e.g. `F?`/`?F`, _not supported_) _in all readable memory
  mappings_.
- `u[bwdqBWDQfF] <constraints>`: Rescan the address list from the last scan for
  nev values.
- `d`: List addresses from the last scan not present in the scan before.
- `d[bwdqBWDQfF] <addr> [<length>]`: Display memory bytes.

# Pointer Highlighting

Certain shown values and addresses are colorized to help _identify_ (but not
validate/verify) useful pointers:
1. Valid pointers to readable memory (when displaying memory bytes with the
   display command) -- helps quickly identify pointer chains and structures.
2. File-backed pointers (when displaying scan results) -- usually static or
   stable pointers.

# Scan heuristics

The scan commands use several _simple_ heuristics to optimize performance and
reduce noise when searching process memory (this is __note__ the case for IDA
byte pattern scanning which scans _all readable memory_). As such, some regions
that are unlikely to contain user data are filtered out when scanning:
* Kernel mappings: `[vvar]`, `[vdso]`, `[vsyscall]`
* System paths: `/dev`, `/sys`, `/proc`
* Special file descriptors: `anon_inode`, `memfd` and deleted files (sections
  ending with `(deleted)` -- these are likely to be caches).
