SECTIONS {
    . = ALIGN(4);

    .command_handlers :
    {
        __start_command_handlers = .;
        KEEP(*(.command_handlers))
        __end_command_handlers = .;
    }
}
INSERT AFTER .text
