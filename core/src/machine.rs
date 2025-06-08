
use crate::*;
use crate::guest::*;
use crate::state::*;
use crate::insn::*;

#[derive(Debug)]
pub struct Machine {
    pub guest: GuestMem,
    pub state: State,
    pub decoders: Vec<Box<dyn Decoder>>,
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

    /// Step the machine, executing a block of instructions.
    /// Loops until a break condition is met. (currently only ecall)
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
            debug!("decoding instruction at {:#x}: {:#x}", self.state.pc, raw);
            let (insn, executor) = match self.decode(raw)? {
                Some((insn, executor)) => (insn, executor),
                None => {
                    error!("unknown instruction at {:#x}: {:#x}", self.state.pc, raw);
                    return Err(Error::Unimplemented);
                }
            };
            debug!("executing instruction: {:?}", insn);

            executor(&mut self.state, &mut self.guest, &insn)?;
            debug!("state after execution: {:?}", self.state);

            match self.state.break_on {
                Some(BreakCause::Ecall) => {
                    debug!("break on ecall at {:#x}", self.state.pc);
                    return Err(Error::Unimplemented);
                },
                Some(BreakCause::Ebreak) => {
                    debug!("break on ebreak at {:#x}", self.state.pc);
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