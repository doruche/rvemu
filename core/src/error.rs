use std::error;

use crate::guest::MemAccess;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidElfHdr,
    InvalidProgHdr,
    SegmentOverlap,
    PermissionDenied,
    MemAccessFault(MemAccess, u64),
    OutOfBounds,
    InternalError(String),
    IoError(std::io::Error),
    Unimplemented,
    SyscallRequired,
    Exit(i64),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidElfHdr => write!(f, "Invalid ELF header"),
            Error::SegmentOverlap => write!(f, "Memory segments overlap"),
            Error::InvalidProgHdr => write!(f, "Invalid program header"),
            Error::PermissionDenied => write!(f, "Permission denied"),
            Error::MemAccessFault(access, gaddr) => write!(f, "Memory access fault: {:?} at {:#x}", access, gaddr),
            Error::OutOfBounds => write!(f, "Memory access out of bounds"),
            Error::SyscallRequired => write!(f, "Syscall required but not implemented"),
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
            Error::Unimplemented => write!(f, "Unimplemented feature or instruction"),
            Error::Exit(code) => write!(f, "Exit with code {}", code),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl error::Error for Error {}