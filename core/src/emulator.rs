//! Interface for users to interact with the emulator.
//! Operations such as loading programs, running them, and accessing the state of the CPU and memory are provided.

use crate::*;
use crate::error::*;
use crate::machine::*;
use crate::state::*;
use crate::syscall::*;

#[derive(Debug)]
pub struct Emulator {
    machine: Machine,
    syscall: Box<dyn SyscallHandler>,
}

pub struct EmulatorBuilder {
    machine: Machine,
    syscall: Option<Box<dyn SyscallHandler>>,
    decoders: Vec<InsnSet>,
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        Self {
            machine: Machine::new(),
            syscall: None,
            decoders: vec![],
        }
    }

    pub fn with_syscall_handler(mut self, handler: Box<dyn SyscallHandler>) -> Self {
        self.syscall = Some(handler);
        self
    }

    pub fn add_decoder(mut self, set: InsnSet) -> Result<Self> {
        self.machine.add_decoder(set)?;
        self.decoders.push(set);
        Ok(self)
    }

    pub fn build(self) -> Result<Emulator> {
        if self.syscall.is_none() {
            return Err(Error::SyscallRequired);
        }
        Ok(Emulator {
            machine: self.machine,
            syscall: self.syscall.unwrap(),
        })
    }
}

impl Emulator {
    pub fn new() -> EmulatorBuilder {
        EmulatorBuilder::new()
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        self.machine.load_program(program)
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.machine.step()? {
                BreakCause::Ecall => {
                    self.syscall.handle(&mut self.machine.state, &mut self.machine.guest)?;
                }
                BreakCause::Ebreak => {
                    return Err(Error::Unimplemented);
                }
            }
        }
    }

    pub fn dump_state(&self) -> String {
        format!("{:#x?}", self.machine.state)
    }

    pub fn state(&self) -> &State {
        &self.machine.state
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
            .with_syscall_handler(Box::new(crate::Mini))
            .add_decoder(InsnSet::I)
            .unwrap()
            .build()
            .unwrap();
        let prog = include_bytes!("../../testprogs/minimal");

        emulator.load_program(prog).unwrap();
        let res = emulator.run();
        match res {
            Err(Error::Exit(code)) => {
                debug!("Program exited with code {}", code);
                debug!("return val {}", emulator.machine.state.x[10]);
                debug!("{}", emulator.dump_state());
            },
            _ => {
                error!("Program did not exit as expected: {:?}", res);
                panic!("Test failed, program did not exit correctly.");
            }
        }
    }

    fn test_inner(test_name: &str) {

        let mut emulator = Emulator::new()
            .with_syscall_handler(Box::new(crate::Mini))
            .add_decoder(InsnSet::I)
            .unwrap()
            .add_decoder(InsnSet::Ziscr)
            .unwrap()
            .add_decoder(InsnSet::Zifencei)
            .unwrap()
            .build()
            .unwrap();

        let mut prog = File::open(format!("../testprogs/riscv-tests/isa/{}", test_name))
            .expect("Failed to open test program file");

        let mut prog_bytes = vec![];
        prog.read_to_end(&mut prog_bytes)
            .expect("Failed to read test program file");
        let prog = prog_bytes.as_slice();

        emulator.load_program(prog).unwrap();
        let res = emulator.run();
        match res {
            Err(Error::Exit(_)) => {
                match emulator.machine.state.x[3] {
                    1 => {
                        debug!("Test {} passed.", test_name);
                        debug!("{}", emulator.dump_state());
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
    fn test() {
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

    #[test]
    fn test_prime() {
        log::log_init(log::Level::Off);

        let mut emulator = Emulator::new()
            .with_syscall_handler(Box::new(crate::Newlib))
            .add_decoder(InsnSet::I)
            .unwrap()
            .build()
            .unwrap();
        let prog = include_bytes!("../../testprogs/prime");

        emulator.load_program(prog).unwrap();
        let res = emulator.run();
        match res {
            Err(Error::Exit(code)) => {
                debug!("Program exited with code {}", code);
                debug!("{}", emulator.dump_state());
            },
            _ => {
                error!("Program did not exit as expected: {:?}", res);
                panic!("Test failed, program did not exit correctly.");
            }
        }
    }
}