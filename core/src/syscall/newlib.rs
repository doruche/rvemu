//! Newlib syscall implementation.
use crate::syscall::*;
use crate::*;
use crate::error::*;
use crate::guest::GuestMem;
use crate::state::State;

#[derive(Debug)]
pub struct NewlibSyscallHandler;

impl SyscallHandler for NewlibSyscallHandler {
    fn handle(&mut self, state: &mut State, guest: &mut GuestMem) -> Result<()> {
        match state.x[17] {
            _ => {
                Err(Error::SyscallUnimplemented(state.x[17], state.pc))
            }
        }
    }
}

impl NewlibSyscallHandler {
    fn sys_exit(&mut self, state: &mut State) -> Result<i64> {
        debug!("sys_exit called with code {}", state.x[10]);
        return Err(Error::Exited(state.x[10] as i64));
    }
}