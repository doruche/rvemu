//! Mini syscall for testing purposes.
use crate::syscall::*;
use crate::error::*;
use crate::guest::GuestMem;
use crate::state::State;
use crate::*;

const SYS_EXIT: u64 = 0;
const SYS_PUTCHAR: u64 = 1;
const SYS_PUTS: u64 = 2;

#[derive(Debug)]
pub struct MinilibSyscallHandler;

impl SyscallHandler for MinilibSyscallHandler {
    fn handle(&mut self, state: &mut State, guest: &mut GuestMem) -> Result<()> {
        match state.x[17] {
            SYS_EXIT|93 => sys_exit(state.x[10] as i64),
            SYS_PUTCHAR => sys_putchar(state.x[10] as u8),
            SYS_PUTS => sys_puts(state, guest, state.x[10]),
            _ => {
                // for testing purposes, we just throw an error for unimplemented syscalls
                Err(Error::SyscallUnimplemented(state.x[17], state.pc))
            }
        }
    }
}

fn sys_exit(exit_code: i64) -> Result<()> {
    debug!("sys_exit called with code {}", exit_code);
    Err(Error::Exit(exit_code))
}

fn sys_putchar(c: u8) -> Result<()> {
    print!("{}", c as char);
    Ok(())
}

fn sys_puts(state: &mut State, guest: &GuestMem, s: u64) -> Result<()> {
    let mut buf = Vec::new();
    let mut ptr = s;

    // Read until null terminator
    loop {
        let byte = match guest.read_u8(ptr) {
            Ok(b) => b,
            Err(Error::MemAccessFault(..)) => {
                warn!("sys_puts: memory access fault at {:#x}", s);
                state.x[0] = u64::MAX;
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        if byte == 0 {
            break;
        }
        buf.push(byte);
        ptr += 1;
    }

    print!("{}", String::from_utf8_lossy(&buf));
    Ok(())
}