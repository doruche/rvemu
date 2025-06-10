//! Currently no-op.

use crate::*;
use crate::error::*;
use crate::guest::*;
use crate::state::State;
use crate::insn::*;

pub const ZIFENCEI_INSN: u32 = 0x0000100f;

#[derive(Debug)]
pub struct ZifenceiDecoder;

impl Decoder for ZifenceiDecoder {
    fn decode(&self, raw: u32) -> Result<Option<(Instruction, Executor)>> {
        if raw == ZIFENCEI_INSN {
            let insn = Instruction::R {
                funct7: 0,
                rs2: 0,
                rs1: 0,
                funct3: 0,
                rd: 0,
                opcode: 0b1110011,
                raw,
            };
            Ok(Some((insn, zifencei as Executor)))
        } else {
            Ok(None)
        }
    }
}

fn zifencei(_state: &mut State, _guest: &mut GuestMem, _insn: &Instruction) -> Result<()> {
    Ok(())
}