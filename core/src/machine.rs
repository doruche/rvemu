
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
            InsnSet::I32 => Box::new(insn::Rv32IDecoder),
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
                        
            // For compressed instructions, we only consume 16 bits.
            if self.state.pc % 2 != 0 {
                error!("PC is not aligned to instruction size at {:#x}", self.state.pc);
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
            self.state.pc += insn.step_size() as u64;

            if let Some(break_cause) = self.state.break_on {
                debug!("break condition met: {:?}", break_cause);
                if matches!(break_cause, BreakCause::Ecall) {
                    debug!("ecall encountered");
                    return Ok(break_cause);
                } else {
                    continue;
                }
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
        m.add_decoder(InsnSet::I32).unwrap();
        let program = include_bytes!("../../testprogs/prime");
        m.load_program(program).unwrap();

        // fake some lui instruction at the start
        // lui x0, 0x37
        m.guest.write_u32(m.state.pc, 0x00000037).unwrap();
        // lui x7, 0x777
        m.guest.write_u32(m.state.pc + 4, 0x003093B7).unwrap();

        let result = m.step();
        debug!("step result: {:#?}", result);
    }
}