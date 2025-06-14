pub const STACK_SIZE: usize = 0x00800000; // 8 MiB

/// Interval to poll for events in the event loop
pub const POLL_INTERVAL: usize = 1024; // 1024 instructions

/// Default gdb port
pub const GDB_PORT: u16 = 3777;

/// Bad address error
pub const EFAULT: u8 = 14;