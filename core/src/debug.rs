use std::net::TcpListener;
use std::net::TcpStream;

use bitflags::parser::to_writer;
use gdbstub::common::Signal;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::run_blocking;
use gdbstub::stub::run_blocking::Event;
use gdbstub::stub::DisconnectReason;
use gdbstub::stub::GdbStub;
use gdbstub::stub::SingleThreadStopReason;
use gdbstub::target::ext::base::single_register_access::SingleRegisterAccess;
use gdbstub::target::ext::base::singlethread::SingleThreadBase;
use gdbstub::target::ext::base::singlethread::SingleThreadResume;
use gdbstub::target::ext::base::singlethread::SingleThreadSingleStep;
use gdbstub::target::ext::breakpoints::Breakpoints;
use gdbstub::target::ext::breakpoints::SwBreakpoint;
use gdbstub::target::Target;
use gdbstub::target::TargetError;
use gdbstub::*;
use gdbstub::stub::run_blocking::BlockingEventLoop;


use crate::config::EFAULT;
use crate::config::GDB_PORT;
use crate::config::POLL_INTERVAL;
use crate::*;
use crate::emulator::*;
use crate::guest::*;
use crate::insn::*;
use crate::error::*;
use crate::hart::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchMode {
    Read,
    Write,
    Access,
}

/// Currently, we only support watchpoints on memory accesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watchpoint {
    addr: u64,
    mode: WatchMode,
}

impl Emulator {
    pub fn read_u8(&self, gaddr: u64) -> Result<u8> {
        self.guest.read_u8(gaddr)
    }

    pub fn write_u8(&mut self, gaddr: u64, value: u8) -> Result<()> {
        self.guest.write_u8(gaddr, value)
    }

    pub fn set_breakpoint(&mut self, gaddr: u64) -> Result<()> {
        if self.breakpoints.contains(&gaddr) {
            return Err(Error::RepeatedBreakpoint(gaddr));
        }
        self.breakpoints.insert(gaddr);
        Ok(())
    }

    pub fn rm_breakpoint(&mut self, gaddr: u64) -> Result<()> {
        if !self.breakpoints.remove(&gaddr) {
            return Err(Error::BreakpointNotFound(gaddr));
        }
        Ok(())
    }

    pub fn set_watchpoint(&mut self, gaddr: u64, mode: WatchMode) -> Result<()> {
        if self.watchpoints.contains(&gaddr) {
            return Err(Error::RepeatedWatchpoint(gaddr));
        }
        self.watchpoints.insert(gaddr);
        Ok(())
    }

    pub fn rm_watchpoint(&mut self, gaddr: u64) -> Result<()> {
        if !self.watchpoints.remove(&gaddr) {
            return Err(Error::WatchpointNotFound(gaddr));
        }
        Ok(())
    }

    /// Start a gdb session for debugging.
    pub fn debug(&mut self) -> Result<()> {
        fn wait_for_tcp(port: u16) -> Result<TcpStream> {
            let sockaddr = format!("127.0.0.1:{}", port);
            eprintln!("Waiting for GDB to connect on {}", sockaddr);
            let socket = TcpListener::bind(sockaddr)
                .map_err(|e| Error::IoError(e, "Failed to bind to TCP socket".to_string()))?;
            let (stream, addr) = socket.accept()
                .map_err(|e| Error::IoError(e, "Failed to accept TCP connection".to_string()))?;
            eprintln!("GDB connected from {}", addr);
            Ok(stream)
        }
        
        let conn: Box<dyn ConnectionExt<Error = std::io::Error>> = Box::new(wait_for_tcp(GDB_PORT)?);
        let gdb = GdbStub::new(conn);

        match gdb.run_blocking::<EventLoop>(self) {
            Ok(disconnect_reason) => match disconnect_reason {
                DisconnectReason::Disconnect => {
                    eprintln!("GDB session disconnected. Running to completion...");
                    self.mode = EmuMode::Run;
                },
                DisconnectReason::TargetExited(code) =>{
                    println!("GDB session exited with code: {}", code);
                },
                DisconnectReason::TargetTerminated(sig) => {
                    println!("GDB session terminated with signal: {:?}", sig);
                }
                _ => unimplemented!(),
            },
            Err(e) => {
                if e.is_target_error() {
                    eprintln!("Target error: {}", e);
                } else if e.is_connection_error() {
                    let (e, kind) = e.into_connection_error().unwrap();
                    eprintln!("Connection error: {} ({:?})", e, kind);
                } else {
                    eprintln!("Unexpected error: {}", e);
                }
            }
        }

        Ok(())
    }

    pub fn run_debug(&mut self, mut poller: impl FnMut() -> bool) -> Result<ExitReason> {
        let mut cycles = 0;
        let mut first_step = true;
        loop {
            match self.mode {
                EmuMode::Debug(ExecMode::Continue) => {
                    match self.step() {
                        Ok(ExitReason::BreakpointHit(addr)) => {
                            if first_step {
                                first_step = false;
                                self.force_step()?;
                                cycles += 1;
                                if cycles % POLL_INTERVAL == 0 {
                                    if poller() {
                                        return Ok(ExitReason::IncomingData);
                                    }
                                }
                            } else {
                                return Ok(ExitReason::BreakpointHit(addr));
                            }
                        },
                        Ok(ExitReason::DoneStep) => {
                            if first_step {
                                first_step = false;
                            }
                            cycles += 1;
                            if cycles % POLL_INTERVAL == 0 {
                                if poller() {
                                    return Ok(ExitReason::IncomingData);
                                }
                            }
                        },
                        Ok(_) => unreachable!(),
                        Err(Error::Exited(code)) => {
                            return Ok(ExitReason::Exited(code));
                        },
                        Err(e) => {
                            return Err(e);
                        }
                    }
                },
                EmuMode::Debug(ExecMode::Step) => {
                    debug!("herer");
                },
                _ => unreachable!(),
            }
        }
    }

}

impl Target for Emulator {
    type Arch = gdbstub_arch::riscv::Riscv64;

    type Error = Error;

    #[inline(always)]
    fn base_ops(&mut self) -> target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        target::ext::base::BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for Emulator {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as arch::Arch>::Registers,
    ) -> target::TargetResult<(), Self> {
        for (i, &x) in self.hart.state.x.iter().enumerate() {
            regs.x[i] = x;
        }
        regs.pc = self.hart.state.pc;
        debug!("Read registers: {:?}", regs);

        Ok(())
    }

    fn write_registers(
        &mut self, 
        regs: &<Self::Arch as arch::Arch>::Registers
    ) -> target::TargetResult<(), Self> {
        for (i, &x) in regs.x.iter().enumerate() {
            self.hart.state.x[i] = x;
        }
        self.hart.state.pc = regs.pc;
        debug!("Wrote registers: {:?}", self.hart.state);
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as arch::Arch>::Usize,
        data: &mut [u8],
    ) -> target::TargetResult<usize, Self> {
        for (i, byte) in data.iter_mut().enumerate() {
            debug!("reading");
            let b = self.guest.read_u8(start_addr + i as <Self::Arch as arch::Arch>::Usize);
            match b {
                Ok(val) => *byte = val,
                Err(e) => if i > 0 {
                    debug!("Read {} bytes before error at address 0x{:x}", i, start_addr + i as u64);
                    return Ok(i);
                } else {
                    debug!("Failed to read byte at address 0x{:x}: {}", start_addr + i as u64, e);
                    return Err(e.into());
                },
            }
        }
        debug!("Read {} bytes from address 0x{:x}", data.len(), start_addr);
        
        Ok(data.len())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as arch::Arch>::Usize,
        data: &[u8],
    ) -> target::TargetResult<(), Self> {
        for (i, &byte) in data.iter().enumerate() {
            self.guest.write_u8(start_addr + i as u64, byte)?;
        }

        debug!("Wrote {} bytes to address 0x{:x}", data.len(), start_addr);
        Ok(())
    }

    #[inline(always)]
    fn support_single_register_access(&mut self)
        -> Option<target::ext::base::single_register_access::SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }

    #[inline(always)]
    fn support_resume(&mut self) 
        -> Option<target::ext::base::singlethread::SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl Breakpoints for Emulator {
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SingleRegisterAccess<()> for Emulator {
    fn read_register(
        &mut self,
        tid: (),
        reg_id: <Self::Arch as arch::Arch>::RegId,
        buf: &mut [u8],
    ) -> target::TargetResult<usize, Self> {
        match reg_id {
            gdbstub_arch::riscv::reg::id::RiscvRegId::Gpr(id) =>  {
                debug!("Reading GPR {}: {}", id, self.hart.state.x[id as usize]);
                buf.copy_from_slice(&self.hart.state.x[id as usize].to_le_bytes());
                Ok(8)
            },
            gdbstub_arch::riscv::reg::id::RiscvRegId::Pc => {
                buf.copy_from_slice(&self.hart.state.pc.to_le_bytes());
                Ok(8)
            },
            _ => Err(TargetError::NonFatal),
        }
    }

    fn write_register(
        &mut self,
        tid: (),
        reg_id: <Self::Arch as arch::Arch>::RegId,
        val: &[u8],
    ) -> target::TargetResult<(), Self> {
        match reg_id {
            gdbstub_arch::riscv::reg::id::RiscvRegId::Gpr(id) => {
                let value = u64::from_le_bytes(val.try_into().unwrap());
                self.hart.state.x[id as usize] = value;
                Ok(())
            },
            gdbstub_arch::riscv::reg::id::RiscvRegId::Pc => {
                self.hart.state.pc = u64::from_le_bytes(val.try_into().unwrap());
                Ok(())
            },
            _ => Err(TargetError::NonFatal),
        }
    }
}

impl SingleThreadResume for Emulator {
    fn resume(&mut self, signal: Option<common::Signal>) -> std::result::Result<(), Self::Error> {
        if signal.is_some() {
            return Err(Error::InternalError("Signal not supported".to_string()));
        }

        self.mode = EmuMode::Debug(ExecMode::Continue);
        
        Ok(())
    }

    fn support_single_step(&mut self) -> Option<target::ext::base::singlethread::SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for Emulator {
    fn step(&mut self, signal: Option<Signal>) -> std::result::Result<(), Self::Error> {
        if signal.is_some() {
            return Err(Error::InternalError("Signal not supported".to_string()));
        }

        self.mode = EmuMode::Debug(ExecMode::Step);
        
        Ok(())
    }
}

impl SwBreakpoint for Emulator {
    fn add_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as arch::Arch>::Usize,
        kind: <Self::Arch as arch::Arch>::BreakpointKind,
    ) -> target::TargetResult<bool, Self> {
        self.set_breakpoint(addr)
            .map(|_| true)
            .map_err(|e| e.into())
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as arch::Arch>::Usize,
        kind: <Self::Arch as arch::Arch>::BreakpointKind,
    ) -> target::TargetResult<bool, Self> {
        self.rm_breakpoint(addr)
            .map(|_| true)
            .map_err(|e| e.into())
    }
}

pub struct EventLoop {}

impl BlockingEventLoop for EventLoop {
    type Target = Emulator;

    type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;

    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> std::result::Result<
        stub::run_blocking::Event<Self::StopReason>,
        stub::run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as conn::Connection>::Error,
        >,
    > {
        let poller = || {
            conn.peek().map(|b| b.is_some()).unwrap_or(false)
        };

        let stop_reason = match target.run_debug(poller) {
            Ok(o) => match o {
                ExitReason::DoneStep => SingleThreadStopReason::DoneStep,
                ExitReason::IncomingData => {
                    let byte = conn
                        .read()
                        .map_err(|e| {
                            debug!("Failed to read byte: {}", e);
                            run_blocking::WaitForStopReasonError::Connection(e)
                        })?;
                    return Ok(Event::IncomingData(byte));
                },
                ExitReason::Exited(code) => {
                    SingleThreadStopReason::Terminated(Signal::SIGSTOP)
                },
                ExitReason::BreakpointHit(addr) => {
                    SingleThreadStopReason::SwBreak(())
                },
            },
            Err(e) => {
                return Err(run_blocking::WaitForStopReasonError::Target(e.into()))
            },
        };

        Ok(Event::TargetStopped(stop_reason))
    }

    fn on_interrupt(
        target: &mut Self::Target,
    ) -> std::result::Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}

impl From<Error> for TargetError<Error> {
    fn from(value: Error) -> Self {
        match value {
            Error::InternalError(_) => Self::Fatal(value),
            Error::MemAccessFault(_, _) => Self::Errno(EFAULT),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        log::log_init(log::Level::Trace);

        let mut emu = Emulator::new()
            .decoder(InsnSet::I)
            .syscall(Box::new(Minilib))
            .debug()
            .build()
            .unwrap();

        emu.load_elf(include_bytes!("../../testprogs/minimal"))
            .unwrap();
        emu.debug().unwrap();
    }
}