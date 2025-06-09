//! Newlib syscall implementation.
use crate::syscall::*;
use crate::*;
use crate::error::*;
use crate::guest::GuestMem;
use crate::state::State;

#[derive(Debug)]
pub struct NewlibSyscallHandler;

impl SyscallHandler for NewlibSyscallHandler {
    fn handle(&mut self, state: &mut State, guest: &mut GuestMem) -> Result<u64> {
        match state.x[17] {
            _ => {
                error!("newlib syscall unimplemented: {}", state.x[17]);
                Err(Error::Unimplemented)
            }
        }
    }
}

impl NewlibSyscallHandler {
    fn sys_exit(&mut self, state: &mut State) -> Result<u64> {
        debug!("sys_exit called with code {}", state.x[10]);
        return Err(Error::Exit(state.x[10] as i64));
    }
}