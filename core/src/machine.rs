
use crate::config::STACK_SIZE;
use crate::*;
use crate::guest::*;
use crate::state::*;
use crate::insn::*;

#[derive(Debug)]
pub struct Machine {
    pub guest: GuestMem,
    pub state: State,
    pub decoders: Vec<Box< dyn Decoder>>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            guest: GuestMem::new(),
            state: State::default(),
            decoders: vec![],
        }
    }

    pub fn add_decoder(&mut self, set: InsnSet) -> Result<()> {
        let decoder = match set {
            InsnSet::I => Box::new(insn::Rv64IDecoder),
            _ => return Err(Error::Unimplemented),
        };
        self.decoders.push(decoder);
        
        Ok(())
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        let entry = self.guest.load_elf(program)?;
        self.state.pc = entry;

        // allocate stack space
        self.guest.add_segment(
            0x8000_0000 - STACK_SIZE as u64,
            STACK_SIZE,
            MemFlags::READ|MemFlags::WRITE,
            None,
        )?;
        self.state.x[2] = 0x8000_0000;
        Ok(())
    }

    pub fn decode(&self, raw: u32) -> Result<Option<(Instruction, Executor)>> {
        for decoder in &self.decoders {
            if let Some((insn, executor)) = decoder.decode(raw)? {
                return Ok(Some((insn, executor)));
            }
        }
        Ok(None)
    }

    pub fn step(&mut self) -> Result<BreakCause> {
        loop {
            self.state.x[0] = 0;
            self.state.break_on = None;

            let cur_pc = self.state.pc;

            // For compressed instructions, we only consume 16 bits.
            if cur_pc % 2 != 0 {
                error!("pc not aligned to instruction size at {:#x}", self.state.pc);
                return Err(Error::InternalError("PC not aligned".to_string()));
            }
            let raw = self.guest.read_u32(self.state.pc)?;
            trace!("decoding instruction at {:#x}: {:#x}", self.state.pc, raw);
            let (insn, executor) = match self.decode(raw)? {
                Some((insn, executor)) => (insn, executor),
                None => {
                    error!("unknown instruction at {:#x}: {:#x}", self.state.pc, raw);
                    return Err(Error::Unimplemented);
                }
            };
            trace!("executing instruction: {:x?}", insn);

            executor(&mut self.state, &mut self.guest, &insn)?;
            trace!("state after execution: {:x?}", self.state);

            match self.state.break_on {
                Some(BreakCause::Ecall) => {
                    trace!("break on ecall at {:#x}", self.state.pc);
                    return Ok(BreakCause::Ecall);
                },
                Some(BreakCause::Ebreak) => {
                    trace!("break on ebreak at {:#x}", self.state.pc);
                    return Err(Error::Unimplemented);
                },
                _ => (),
            }

            if cur_pc == self.state.pc {
                // if pc did not change, it must be a normal instruction, otherwise some branch...
                self.state.pc = cur_pc + insn.step_size() as u64;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_step() {
        log::log_init(log::Level::Off);

        let mut m = Machine::new();
        m.add_decoder(InsnSet::I).unwrap();
        let program = include_bytes!("../../testprogs/minimal");
        m.load_program(program).unwrap();

        let result = m.step();
        debug!("step result: {:#?}", result);
    }
}