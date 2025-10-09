crate::register_command_handler!(
    handler, ["db", "dw", "dd", "dq", "dB", "dW", "dD", "dQ", "df", "dF"],
    "Display memory bytes.",
r#"`<address> [length]`
* `address` -- The address where the display should start
* `length` -- The amount of bytes to show. `64` by default.
"#
);

fn handler(s: &mut crate::Scanner, args: &[String]) -> crate::commands::Result {
    Err("Displaying..".to_string())
}
