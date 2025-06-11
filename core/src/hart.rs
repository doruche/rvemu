
use std::sync::Arc;

use crate::config::STACK_SIZE;
use crate::*;
use crate::guest::*;
use crate::state::*;
use crate::insn::*;

#[derive(Debug)]
pub struct Hart {
    pub state: State,
    pub decoders: Vec<Arc<dyn Decoder>>,
}

impl Hart {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            decoders: vec![],
        }
    }

    pub fn add_decoder(&mut self, set: InsnSet) -> Result<()> {
        let decoder: Arc<dyn Decoder> = match set {
            InsnSet::I => Arc::new(insn::Rv64IDecoder),
            InsnSet::Zifencei => Arc::new(insn::ZifenceiDecoder),
            InsnSet::Ziscr => Arc::new(insn::ZicsrDecoder),
            _ => return Err(Error::Unimplemented),
        };
        self.decoders.push(decoder);
        
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

    pub fn step(&mut self, guest: &mut GuestMem) -> Result<Option<BreakCause>> {
        self.state.x[0] = 0;
        self.state.break_on = None;

        let cur_pc = self.state.pc;
        // For compressed instructions, we only consume 16 bits.
        if cur_pc % 2 != 0 {
            error!("pc not aligned to instruction size at {:#x}", self.state.pc);
            return Err(Error::InternalError("PC not aligned".to_string()));
        }
        
        let raw = guest.read_u32(self.state.pc)?;
        let (insn, executor) = match self.decode(raw)? {
            Some((insn, executor)) => (insn, executor),
            None => {
                error!("unknown instruction at {:#x}: {:#x}", self.state.pc, raw);
                return Err(Error::Unimplemented);
            }
        };

        trace!("pc@{:#x}: executing instruction: {:x?}", self.state.pc, insn);
        trace!("state before: {:x?}", self.state);
        executor(&mut self.state, guest, &insn)?;

        if cur_pc == self.state.pc {
            // if pc did not change, it must be a normal instruction, otherwise some branch...
            self.state.pc = cur_pc + insn.step_size() as u64;
        }
        
        if let Some(cause) = self.state.break_on {
            match cause {
                BreakCause::Ecall => {
                    trace!("break on ecall at {:#x}", self.state.pc);
                },
                BreakCause::Ebreak => {
                    trace!("break on ebreak at {:#x}", self.state.pc);
                },
            }
            return Ok(Some(cause));
        }

        Ok(None)
    }
}