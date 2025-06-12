use std::net::TcpStream;

use gdbstub::stub::SingleThreadStopReason;
use gdbstub::target::ext::base::single_register_access::SingleRegisterAccess;
use gdbstub::target::ext::base::singlethread::SingleThreadBase;
use gdbstub::target::ext::base::singlethread::SingleThreadResume;
use gdbstub::target::ext::breakpoints::Breakpoints;
use gdbstub::target::ext::breakpoints::SwBreakpoint;
use gdbstub::target::Target;
use gdbstub::target::TargetError;
use gdbstub::*;
use gdbstub::stub::run_blocking::BlockingEventLoop;


use crate::config::EFAULT;
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
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as arch::Arch>::Usize,
        data: &mut [u8],
    ) -> target::TargetResult<usize, Self> {
        for (i, byte) in data.iter_mut().enumerate() {
            let b = self.guest.read_u8(start_addr + i as <Self::Arch as arch::Arch>::Usize);
            match b {
                Ok(val) => *byte = val,
                Err(e) => if i > 0 {
                    return Ok(i);
                } else {
                    return Err(e.into());
                },
            }
        }
        
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
        Ok(())
    }

    fn support_single_register_access(&mut self)
        -> Option<target::ext::base::single_register_access::SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }

    fn support_resume(&mut self) 
        -> Option<target::ext::base::singlethread::SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

}

impl Breakpoints for Emulator {
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
        todo!()
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

    type Connection = TcpStream;

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
        todo!()
    }

    fn on_interrupt(
        target: &mut Self::Target,
    ) -> std::result::Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        todo!()
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