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
            Error::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl error::Error for Error {}