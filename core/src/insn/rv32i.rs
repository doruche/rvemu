//! RV32I instruction set architecture

use crate::guest::GuestMem;
use crate::insn::{Decoder, Executor, Instruction};
use crate::state::State;
use crate::*;
use crate::error::*;

pub const RV32I_OPCODE_LOAD: u8 = 0b0000011;
pub const RV32I_OPCODE_STORE: u8 = 0b0100011;
pub const RV32I_OPCODE_OP_IMM: u8 = 0b0010011;
pub const RV32I_OPCODE_OP: u8 = 0b0110011;
pub const RV32I_OPCODE_BRANCH: u8 = 0b1100011;
pub const RV32I_OPCODE_JAL: u8 = 0b1101111;
pub const RV32I_OPCODE_JALR: u8 = 0b1100111;
pub const RV32I_OPCODE_LUI: u8 = 0b0110111;
pub const RV32I_OPCODE_AUIPC: u8 = 0b0010111;
pub const RV32I_OPCODE_FENCE: u8 = 0b0001111;
pub const RV32I_OPCODE_SYSTEM: u8 = 0b1110011;

#[derive(Debug)]
pub struct Rv32IDecoder;

impl Decoder for Rv32IDecoder {
    fn decode(&self, raw: u32) -> Result<Option<(Instruction, Executor)>> {
        let opcode = (raw & 0x7f) as u8;
        let rd = ((raw >> 7) & 0x1f) as u8;
        let funct3 = ((raw >> 12) & 0x07) as u8;
        let rs1 = ((raw >> 15) & 0x1f) as u8;
        let rs2 = ((raw >> 20) & 0x1f) as u8;
        let funct7 = ((raw >> 25) & 0x7f) as u8;

        let res = match opcode {
            RV32I_OPCODE_LUI => (Instruction::U {
                rd,
                opcode,
                raw,
                imm: (raw >> 12) as u32,
            }, rv32i_lui as Executor),
            _ => return Ok(None),
        };

        Ok(Some(res))
    }
}


pub fn rv32i_lui(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    u!(insn, rd, imm => {
        let value = imm << 12;
        state.x[rd as usize] = value;
        Ok(())
    })
}