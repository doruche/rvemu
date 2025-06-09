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
}

#[cfg(test)]
mod tests {
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