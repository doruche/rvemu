//! Mini syscall for testing purposes.
//! 0 - exit

use crate::syscall::*;
use crate::error::*;
use crate::guest::GuestMem;
use crate::state::State;
use crate::*;

#[derive(Debug)]
pub struct MiniSyscallHandler;

impl SyscallHandler for MiniSyscallHandler {
    fn handle(&mut self, state: &mut State, guest: &mut GuestMem) -> Result<u64> {
        match state.x[17] {
            0 => sys_exit(state),
            _ => {
                error!("mini syscall unimplemented: {}", state.x[17]);
                Err(Error::Unimplemented)
            }
        }
    }
}

fn sys_exit(state: &mut State) -> Result<u64> {
    debug!("sys_exit called with code {}", state.x[10]);
    Err(Error::Exit(state.x[10] as i64))
}