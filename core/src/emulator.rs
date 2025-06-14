//! Interface for users to interact with the emulator.
//! Operations such as loading programs, running them, and accessing the state of the CPU and memory are provided.

use std::collections::HashSet;
use std::net::TcpListener;
use std::net::TcpStream;

use gdbstub::conn::ConnectionExt;
use gdbstub::stub::GdbStub;

use crate::debug::WatchMode;
use crate::guest::*;
use crate::insn::*;
use crate::*;
use crate::config::*;
use crate::error::*;
use crate::hart::*;
use crate::state::*;
use crate::syscall::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmuMode {
    Run,
    Debug(ExecMode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecMode {
    Step,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitReason {
    DoneStep,
    IncomingData,
    Exited(i64),
    BreakpointHit(u64),
}

pub struct Emulator {
    // harts: Vec<Hart>,
    pub(crate) hart: Hart,
    // guest: Arc<RwLock<GuestMem>>,
    pub(crate) guest: GuestMem,
    pub(crate) syscall: Box<dyn SyscallHandler>,
    pub(crate) stack_size: usize,
    pub(crate) breakpoints: HashSet<u64>,
    pub(crate) watchpoints: HashSet<u64>,
    pub(crate) mode: EmuMode,
    pub(crate) isa: Vec<InsnSet>,
}

pub struct EmulatorBuilder {
    hart: Hart,
    syscall: Option<Box<dyn SyscallHandler>>,
    decoders: Vec<InsnSet>,
    /// default stack size in bytes (8 MiB)
    stack_size: usize,
    mode: EmuMode,
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        Self {
            hart: Hart::new(0),
            syscall: None,
            decoders: vec![],
            stack_size: STACK_SIZE,
            mode: EmuMode::Run,
        }
    }

    pub fn syscall(mut self, handler: Box<dyn SyscallHandler>) -> Self {
        self.syscall = Some(handler);
        self
    }

    pub fn decoder(mut self, set: InsnSet) -> Self {
        self.decoders.push(set);
        self
    }

    pub fn stack_size(mut self, size: usize) -> Self {
        self.stack_size = size;
        self
    }

    pub fn debug(mut self) -> Self {
        self.mode = EmuMode::Debug(ExecMode::Step);
        self
    }

    pub fn build(mut self) -> Result<Emulator> {
        if self.syscall.is_none() {
            return Err(Error::Other("Syscall handler not set".to_string()));
        }
        let mut isa = vec![];
        for set in self.decoders.iter() {
            self.hart.add_decoder(*set)?;
            isa.push(*set);
        }
        Ok(Emulator {
            hart: self.hart,
            guest: GuestMem::new(),
            syscall: self.syscall.unwrap(),
            stack_size: self.stack_size,
            breakpoints: HashSet::new(),
            watchpoints: HashSet::new(),
            mode: self.mode,
            isa,
        })
    }
}

impl Emulator {
    pub fn new() -> EmulatorBuilder {
        EmulatorBuilder::new()
    }

    pub fn load_elf(&mut self, program: &[u8]) -> Result<()> {
        let entry = self.guest.load_elf(program)?;
        self.hart.state.pc = entry;

        // allocate stack space
        self.guest.add_segment(
            0x8000_0000 - self.stack_size as u64,
            self.stack_size,
            0x1000,
            MemFlags::READ|MemFlags::WRITE,
            None,
        )?;
        self.hart.state.x[2] = 0x8000_0000;
        Ok(())
    }

    pub fn run(&mut self) -> Result<ExitReason> {
        match self.mode {
            EmuMode::Run => {
                loop {
                    match self.force_step() {
                        Ok(_) => {},
                        Err(Error::Exited(code)) => {
                            return Ok(ExitReason::Exited(code));
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            },
            _ => unreachable!(),
        }
    }

    pub fn step(&mut self) -> Result<ExitReason> {
        if self.breakpoints.contains(&self.hart.state.pc) {
            return Err(Error::BreakpointHit);
        }
        self.force_step()
    }

    pub fn force_step(&mut self) -> Result<ExitReason> {
        match self.hart.step(&mut self.guest)? {
            Some(BreakCause::Ecall) => {
                self.syscall.handle(&mut self.hart.state, &mut self.guest)?;
            }
            Some(BreakCause::Ebreak) => {
                unimplemented!();
            }
            None => {}
        }
        Ok(ExitReason::DoneStep)
    }

}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use super::*;
    
    #[test]
    fn test_minimal() {
        log::log_init(log::Level::Trace);

        let mut emulator = Emulator::new()
            .syscall(Box::new(crate::Minilib))
            .decoder(InsnSet::I)
            .build()
            .unwrap();
        let prog = include_bytes!("../../testprogs/minimal");

        emulator.load_elf(prog).unwrap();
        let res = emulator.run();
        match res {
            Ok(_) => {},
            Err(e) => {
                error!("Program did not exit as expected: {:?}", e);
            }
        }
    }

    fn test_inner(test_name: &str) {

        let mut emulator = Emulator::new()
            .syscall(Box::new(crate::Minilib))
            .decoder(InsnSet::I)
            .decoder(InsnSet::Ziscr)
            .decoder(InsnSet::Zifencei)
            .build()
            .unwrap();

        let mut prog = File::open(format!("../testprogs/riscv-tests/isa/{}", test_name))
            .expect("Failed to open test program file");

        let mut prog_bytes = vec![];
        prog.read_to_end(&mut prog_bytes)
            .expect("Failed to read test program file");
        let prog = prog_bytes.as_slice();

        emulator.load_elf(prog).unwrap();
        let res = emulator.run();
        match res {
            Ok(_) => {
                match emulator.hart.state.x[3] {
                    1 => {
                        debug!("Test {} passed.", test_name);
                    },
                    _ => {
                        panic!("Test {} failed", test_name);
                    }
                }
            },
            _ => {
                error!("Program did not exit as expected: {:?}", res);
                panic!("Test failed, program did not exit correctly.");
            }
        }

    }

    #[test]
    fn test_rv64i() {
        log::log_init(log::Level::Trace);

        // 53/54; fence_i
        // test_inner("rv64ui-p-add");
        // test_inner("rv64ui-p-addi");
        // test_inner("rv64ui-p-addiw");
        // test_inner("rv64ui-p-addw");
        // test_inner("rv64ui-p-and");
        // test_inner("rv64ui-p-andi");
        // test_inner("rv64ui-p-auipc");
        // test_inner("rv64ui-p-beq");
        // test_inner("rv64ui-p-bge");
        // test_inner("rv64ui-p-bgeu");
        // test_inner("rv64ui-p-blt");
        // test_inner("rv64ui-p-bltu");
        // test_inner("rv64ui-p-bne");
        // test_inner("rv64ui-p-fence_i");
        // test_inner("rv64ui-p-jal");
        // test_inner("rv64ui-p-jalr");
        // test_inner("rv64ui-p-lb");
        // test_inner("rv64ui-p-lbu");
        // test_inner("rv64ui-p-ld_st");
        // test_inner("rv64ui-p-lh");
        // test_inner("rv64ui-p-lhu");
        // test_inner("rv64ui-p-lui");
        // test_inner("rv64ui-p-lw");
        // test_inner("rv64ui-p-lwu");
        // test_inner("rv64ui-p-ld");
        // test_inner("rv64ui-p-ma_data");
        // test_inner("rv64ui-p-or");
        // test_inner("rv64ui-p-ori");
        // test_inner("rv64ui-p-sb");
        // test_inner("rv64ui-p-sd");
        // test_inner("rv64ui-p-sh");
        // test_inner("rv64ui-p-simple");
        // test_inner("rv64ui-p-sll");
        // test_inner("rv64ui-p-slli");
        // test_inner("rv64ui-p-slliw");
        // test_inner("rv64ui-p-sllw");
        // test_inner("rv64ui-p-slt");
        // test_inner("rv64ui-p-slti");
        // test_inner("rv64ui-p-sltiu");
        // test_inner("rv64ui-p-sltu");
        // test_inner("rv64ui-p-sra");
        // test_inner("rv64ui-p-srai");
        // test_inner("rv64ui-p-sraiw");
        // test_inner("rv64ui-p-sraw");
        // test_inner("rv64ui-p-srl");
        // test_inner("rv64ui-p-srli");
        // test_inner("rv64ui-p-srliw");
        // test_inner("rv64ui-p-srlw");
        // test_inner("rv64ui-p-st_ld");
        // test_inner("rv64ui-p-sub");
        // test_inner("rv64ui-p-subw");
        // test_inner("rv64ui-p-sw");
        // test_inner("rv64ui-p-xor");
        // test_inner("rv64ui-p-xori");
    }
}