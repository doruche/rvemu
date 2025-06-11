//! Interface for users to interact with the emulator.
//! Operations such as loading programs, running them, and accessing the state of the CPU and memory are provided.

use std::sync::Arc;

use crate::guest::*;
use crate::insn::*;
use crate::*;
use crate::config::*;
use crate::error::*;
use crate::hart::*;
use crate::state::*;
use crate::syscall::*;

#[derive(Debug)]
pub struct Emulator {
    // harts: Vec<Hart>,
    hart: Hart,
    // guest: Arc<RwLock<GuestMem>>,
    guest: GuestMem,
    syscall: Box<dyn SyscallHandler>,
    stack_size: usize,
    isa: Vec<InsnSet>,
}

pub struct EmulatorBuilder {
    hart: Hart,
    syscall: Option<Box<dyn SyscallHandler>>,
    decoders: Vec<InsnSet>,
    /// default stack size in bytes (8 MiB)
    stack_size: usize,
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        Self {
            hart: Hart::new(0),
            syscall: None,
            decoders: vec![],
            stack_size: STACK_SIZE,
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

    pub fn build(mut self) -> Result<Emulator> {
        if self.syscall.is_none() {
            return Err(Error::SyscallRequired);
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
            MemFlags::READ|MemFlags::WRITE,
            None,
        )?;
        self.hart.state.x[2] = 0x8000_0000;
        Ok(())
    }

    pub fn run(&mut self) -> Result<i64> {
        loop {
            match self.step() {
                Ok(_) => {},
                Err(Error::Exit(code)) => {
                    debug!("Program exited with code {}", code);
                    return Ok(code);
                }
                Err(e) => {
                    error!("Error during execution: {:?}", e);
                    return Err(e);
                }
            }
        }
    }

    pub fn step(&mut self) -> Result<()> {
        match self.hart.step(&mut self.guest)? {
            Some(BreakCause::Ecall) => {
                self.syscall.handle(&mut self.hart.state, &mut self.guest)?;
            }
            Some(BreakCause::Ebreak) => {
                return Err(Error::Unimplemented);
            }
            None => {}
        }
        Ok(())
    }

    pub fn state(&self) -> &State {
        &self.hart.state
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
            Err(Error::Exit(code)) => {
                debug!("Program exited with code {}", code);
                debug!("return val {}", emulator.hart.state.x[10]);
            },
            _ => {
                error!("Program did not exit as expected: {:?}", res);
                panic!("Test failed, program did not exit correctly.");
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
            Err(Error::Exit(_)) => {
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
        // log::log_init(log::Level::Trace);

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