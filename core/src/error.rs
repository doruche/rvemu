use std::error;

use crate::{guest::MemAccess, InsnSet};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidElf,
    MemAccessFault(MemAccess, u64),
    StackOverflow,
    IoError(std::io::Error, String),
    InsnSetUnimplemented(InsnSet),
    /// Used when building a new instruction set
    InsnUnimplemented(u32),
    /// (insn, pc)
    UnknownInsn(u32, u64),
    /// (syscall, pc)
    SyscallUnimplemented(u64, u64),
    Other(String),
    InternalError(String),

    // Debug
    RepeatedBreakpoint(u64),
    RepeatedWatchpoint(u64),
    BreakpointNotFound(u64),
    WatchpointNotFound(u64),

    // Control flow exceptions
    Exit(i64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidElf => write!(f, "Invalid ELF file"),
            Error::MemAccessFault(access, gaddr) => write!(f, "Memory access fault: {:?} at {:#x}", access, gaddr),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
            Error::InsnSetUnimplemented(set) => write!(f, "Instruction set unimplemented: {:?}", set),
            Error::InsnUnimplemented(insn) => write!(f, "Instruction unimplemented: {:#x}", insn),
            Error::UnknownInsn(insn, pc) => write!(f, "Unknown instruction: {:#x} at {:#x}", insn, pc),
            Error::SyscallUnimplemented(syscall, pc) => write!(f, "Syscall unimplemented: {} at {:#x}", syscall, pc),
            Error::RepeatedBreakpoint(addr) => write!(f, "Repeated breakpoint at {:#x}", addr),
            Error::RepeatedWatchpoint(addr) => write!(f, "Repeated watchpoint at {:#x}", addr),
            Error::BreakpointNotFound(addr) => write!(f, "Breakpoint not found at {:#x}", addr),
            Error::WatchpointNotFound(addr) => write!(f, "Watchpoint not found at {:#x}", addr),
            Error::Exit(code) => write!(f, "Exit with code {}", code),
            Error::IoError(err, path) => {
                let msg = err.to_string();
                if path.is_empty() {
                    write!(f, "I/O error: {}", msg)
                } else {
                    write!(f, "I/O error on '{}': {}", path, msg)
                }
            }
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl error::Error for Error {}