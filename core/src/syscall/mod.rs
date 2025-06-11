use std::fmt::Debug;

use crate::guest::GuestMem;
use crate::state::State;
use crate::*;
use crate::error::*;

pub mod minilib;
pub mod newlib;

pub use newlib::NewlibSyscallHandler as Newlib;
pub use minilib::MinilibSyscallHandler as Minilib;

pub trait SyscallHandler: Debug {
    fn handle(&mut self, state: &mut State, guest: &mut GuestMem) -> Result<()>;
}