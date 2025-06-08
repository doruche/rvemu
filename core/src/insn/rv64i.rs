//! RV32I instruction set architecture

use crate::guest::GuestMem;
use crate::insn::{Decoder, Executor, InsnType, Instruction};
use crate::state::{BreakCause, State};
use crate::*;
use crate::error::*;

pub const RV64I_OPCODE_LOAD: u8 = 0b0000011;
pub const RV64I_OPCODE_STORE: u8 = 0b0100011;
pub const RV64I_OPCODE_OP_IMM: u8 = 0b0010011;
pub const RV64I_OPCODE_OP: u8 = 0b0110011;
pub const RV64I_OPCODE_BRANCH: u8 = 0b1100011;
pub const RV64I_OPCODE_WORD: u8 = 0b0111011;
pub const RV64I_OPCODE_JAL: u8 = 0b1101111;
pub const RV64I_OPCODE_JALR: u8 = 0b1100111;
pub const RV64I_OPCODE_LUI: u8 = 0b0110111;
pub const RV64I_OPCODE_AUIPC: u8 = 0b0010111;
pub const RV64I_OPCODE_FENCE: u8 = 0b0001111;
pub const RV64I_OPCODE_SYSTEM: u8 = 0b1110011;

#[derive(Debug)]
pub struct Rv64IDecoder;

impl Decoder for Rv64IDecoder {
    fn decode(&self, raw: u32) -> Result<Option<(Instruction, Executor)>> {
        let opcode = (raw & 0x7f) as u8;
        let rd = ((raw >> 7) & 0x1f) as u8;
        let funct3 = ((raw >> 12) & 0x07) as u8;
        let rs1 = ((raw >> 15) & 0x1f) as u8;
        let rs2 = ((raw >> 20) & 0x1f) as u8;
        let funct7 = ((raw >> 25) & 0x7f) as u8;

        let imm_i = Instruction::extract_imm(raw, InsnType::I);
        let imm_s = Instruction::extract_imm(raw, InsnType::S);
        let imm_b = Instruction::extract_imm(raw, InsnType::B);
        let imm_u = Instruction::extract_imm(raw, InsnType::U);
        let imm_j = Instruction::extract_imm(raw, InsnType::J);

        let res = match opcode {
            RV64I_OPCODE_LUI => (Instruction::U {
                rd,
                opcode,
                raw,
                imm: imm_u,
            }, rv64i_lui as Executor),
            RV64I_OPCODE_AUIPC => (Instruction::U {
                rd,
                opcode,
                raw,
                imm: imm_u,
            }, rv64i_auipc as Executor),
            RV64I_OPCODE_LOAD => match funct3 {
                0b000 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_lb as Executor),
                0b001 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_lh as Executor),
                0b010 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_lw as Executor),
                0b011 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_ld as Executor),
                0b100 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_lbu as Executor),
                0b101 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_lhu as Executor),
                0b110 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_lwu as Executor),
                _ => return Ok(None),
            },
            RV64I_OPCODE_STORE => match funct3 {
                0b000 => (Instruction::S {
                    rs2,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_s,
                }, rv64i_sb as Executor),
                0b001 => (Instruction::S {
                    rs2,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_s,
                }, rv64i_sh as Executor),
                0b010 => (Instruction::S {
                    rs2,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_s,
                }, rv64i_sw as Executor),
                0b011 => (Instruction::S {
                    rs2,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_s,
                }, rv64i_sd as Executor),
                _ => return Ok(None),
            },
            RV64I_OPCODE_OP_IMM => match funct3 {
                0b000 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_addi as Executor),
                0b001 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_slli as Executor),
                0b010 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_slti as Executor),
                0b011 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_sltiu as Executor),
                0b100 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_xori as Executor),
                0b101 => match funct7 {
                        0 => (Instruction::I {
                            rd,
                            rs1,
                            funct3,
                            opcode,
                            raw,
                            imm: imm_i,
                        }, rv64i_srli as Executor),
                        0b0100000 => (Instruction::I {
                            rd,
                            rs1,
                            funct3,
                            opcode,
                            raw,
                            imm: imm_i,
                        }, rv64i_srai as Executor),
                        _ => return Ok(None),
                    },
                0b110 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_ori as Executor),
                0b111 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_andi as Executor),
                _ => return Ok(None),
            },
            RV64I_OPCODE_BRANCH => match funct3 {
                0b000 => (Instruction::B {
                    rs1,
                    rs2,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_b,
                }, rv64i_beq as Executor),
                0b001 => (Instruction::B {
                    rs1,
                    rs2,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_b,
                }, rv64i_bne as Executor),
                0b100 => (Instruction::B {
                    rs1,
                    rs2,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_b,
                }, rv64i_blt as Executor),
                0b101 => (Instruction::B {
                    rs1,
                    rs2,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_b,
                }, rv64i_bge as Executor),
                0b110 => (Instruction::B {
                    rs1,
                    rs2,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_b,
                }, rv64i_bltu as Executor),
                0b111 => (Instruction::B {
                    rs1,
                    rs2,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_b,
                }, rv64i_bgeu as Executor),
                _ => return Ok(None),
            },
            RV64I_OPCODE_JAL => (Instruction::J {
                rd,
                opcode,
                raw,
                imm: imm_j,
            }, rv64i_jal as Executor),
            RV64I_OPCODE_JALR => (match funct3 {
                0b000 => (Instruction::I {
                    rd,
                    rs1,
                    funct3,
                    opcode,
                    raw,
                    imm: imm_i,
                }, rv64i_jalr as Executor),
                _ => return Ok(None),
            }),
            RV64I_OPCODE_OP => match funct3 {
                0b000 => match funct7 {
                    0 => (Instruction::R {
                        rd,
                        rs1,
                        rs2,
                        funct3,
                        funct7,
                        opcode,
                        raw,
                    }, rv64i_add as Executor),
                    0b0100000 => (Instruction::R {
                        rd,
                        rs1,
                        rs2,
                        funct3,
                        funct7,
                        opcode,
                        raw,
                    }, rv64i_sub as Executor),
                    _ => return Ok(None),
                },
                0b001 => (Instruction::R {
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                    opcode,
                    raw,
                }, rv64i_sll as Executor),
                0b010 => (Instruction::R {
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                    opcode,
                    raw,
                }, rv64i_slt as Executor),
                0b011 => (Instruction::R {
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                    opcode,
                    raw,
                }, rv64i_sltu as Executor),
                0b100 => (Instruction::R {
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                    opcode,
                    raw,
                }, rv64i_xor as Executor),
                0b101 => match funct7 {
                        0 => (Instruction::R {
                            rd,
                            rs1,
                            rs2,
                            funct3,
                            funct7,
                            opcode,
                            raw,
                        }, rv64i_srl as Executor),
                        0b0100000 => (Instruction::R {
                            rd,
                            rs1,
                            rs2,
                            funct3,
                            funct7,
                            opcode,
                            raw,
                        }, rv64i_sra as Executor),
                        _ => return Ok(None),
                    },
                0b110 => (Instruction::R {
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                    opcode,
                    raw,
                }, rv64i_or as Executor),
                0b111 => (Instruction::R {
                    rd,
                    rs1,
                    rs2,
                    funct3,
                    funct7,
                    opcode,
                    raw,
                }, rv64i_and as Executor),
                _ => return Ok(None),
            },
            RV64I_OPCODE_SYSTEM => unimplemented!(),
            _ => return Ok(None),
        };

        Ok(Some(res))
    }
}


pub fn rv64i_lui(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    u!(insn, rd, imm => {
        state.x[rd as usize] = zero_extend!(imm, 32);
        Ok(())
    })
}

pub fn rv64i_auipc(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    u!(insn, rd, imm => {
        let value = state.pc.wrapping_add(zero_extend!(imm, 32));
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_lb(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = sign_extend!(guest.read_u8(addr)?, 8) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_lbu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = zero_extend!(guest.read_u8(addr)?, 8) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_lh(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = sign_extend!(guest.read_u16(addr)?, 16) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_lhu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = zero_extend!(guest.read_u16(addr)?, 16) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_lw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = zero_extend!(guest.read_u32(addr)?, 32) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_ld(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = guest.read_u64(addr)?;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_sb(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    s!(insn, rs2, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = (state.x[rs2 as usize] & 0xff) as u8;
        guest.write_u8(addr, value)?;
        Ok(())
    })
}

pub fn rv64i_sh(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    s!(insn, rs2, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = (state.x[rs2 as usize] & 0xffff) as u16;
        guest.write_u16(addr, value)?;
        Ok(())
    })
}

pub fn rv64i_sw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    s!(insn, rs2, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = (state.x[rs2 as usize] & 0xffffffff) as u32;
        guest.write_u32(addr, value)?;
        Ok(())
    })
}

pub fn rv64i_sd(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    s!(insn, rs2, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = state.x[rs2 as usize];
        guest.write_u64(addr, value)?;
        Ok(())
    })
}

pub fn rv64i_addi(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12) as u64;
        let value = state.x[rs1 as usize].wrapping_add(imm);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_slli(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = state.x[rs1 as usize] << (imm & 0x1f);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_srli(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = state.x[rs1 as usize] >> (imm & 0x1f);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_srai(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = state.x[rs1 as usize] as i64 >> (imm & 0x1f);
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_xori(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = state.x[rs1 as usize] ^ sign_extend!(imm, 12) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_ori(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = state.x[rs1 as usize] | sign_extend!(imm, 12) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_andi(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = state.x[rs1 as usize] & sign_extend!(imm, 12) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_slti(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = if (state.x[rs1 as usize] as i64) < sign_extend!(imm, 12) { 1 } else { 0 };
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_sltiu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = if state.x[rs1 as usize] < sign_extend!(imm, 12) as u64 { 1 } else { 0 };
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_add(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize].wrapping_add(state.x[rs2 as usize]);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_sub(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize].wrapping_sub(state.x[rs2 as usize]);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_sll(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize] << (state.x[rs2 as usize] & 0x1f);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_srl(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize] >> (state.x[rs2 as usize] & 0x1f);
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_sra(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = (state.x[rs1 as usize] as i64) >> (state.x[rs2 as usize] & 0x1f);
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_xor(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize] ^ state.x[rs2 as usize];
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_or(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize] | state.x[rs2 as usize];
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_and(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = state.x[rs1 as usize] & state.x[rs2 as usize];
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_slt(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = if (state.x[rs1 as usize] as i64) < (state.x[rs2 as usize] as i64) { 1 } else { 0 };
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_sltu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = if state.x[rs1 as usize] < state.x[rs2 as usize] { 1 } else { 0 };
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_beq(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    b!(insn, rs1, rs2, imm => {
        if state.x[rs1 as usize] == state.x[rs2 as usize] {
            state.pc = state.pc.wrapping_add(sign_extend!(imm, 12) as u64);
        }
        Ok(())
    })
}

pub fn rv64i_bne(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    b!(insn, rs1, rs2, imm => {
        if state.x[rs1 as usize] != state.x[rs2 as usize] {
            state.pc = state.pc.wrapping_add(sign_extend!(imm, 12) as u64);
        }
        Ok(())
    })
}

pub fn rv64i_blt(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    b!(insn, rs1, rs2, imm => {
        if (state.x[rs1 as usize] as i64) < (state.x[rs2 as usize] as i64) {
            state.pc = state.pc.wrapping_add(sign_extend!(imm, 12) as u64);
        }
        Ok(())
    })
}

pub fn rv64i_bge(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    b!(insn, rs1, rs2, imm => {
        if (state.x[rs1 as usize] as i64) >= (state.x[rs2 as usize] as i64) {
            state.pc = state.pc.wrapping_add(sign_extend!(imm, 12) as u64);
        }
        Ok(())
    })
}

pub fn rv64i_bltu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    b!(insn, rs1, rs2, imm => {
        if state.x[rs1 as usize] < state.x[rs2 as usize] {
            state.pc = state.pc.wrapping_add(sign_extend!(imm, 12) as u64);
        }
        Ok(())
    })
}

pub fn rv64i_bgeu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    b!(insn, rs1, rs2, imm => {
        if state.x[rs1 as usize] >= state.x[rs2 as usize] {
            state.pc = state.pc.wrapping_add(sign_extend!(imm, 12) as u64);
        }
        Ok(())
    })
}

pub fn rv64i_jal(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    j!(insn, rd, imm => {
        let target = state.pc.wrapping_add(sign_extend!(imm, 21) as u64);
        state.x[rd as usize] = state.pc.wrapping_add(4);
        state.pc = target;
        Ok(())
    })
}

pub fn rv64i_jalr(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let target = (state.x[rs1 as usize] as i64).wrapping_add(sign_extend!(imm, 12)) & !1;
        state.x[rd as usize] = state.pc.wrapping_add(4);
        state.pc = target as u64;
        Ok(())
    })
}

pub fn rv64i_lwu(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12);
        let addr = (state.x[rs1 as usize] as i64).wrapping_add(imm) as u64;
        let value = zero_extend!(guest.read_u32(addr)?, 32) as u64;
        state.x[rd as usize] = value;
        Ok(())
    })
}

pub fn rv64i_addiw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let imm = sign_extend!(imm, 12) as i32;
        let value = sign_extend!(
            (state.x[rs1 as usize] as i32).wrapping_add(imm),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_slliw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as u32) << (imm & 0x1f),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_srliw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as u32) >> (imm & 0x1f),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_sraiw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, rd, rs1, imm => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as i32) >> (imm & 0x1f),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_addw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as u32).wrapping_add(state.x[rs2 as usize] as u32),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_subw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as u32).wrapping_sub(state.x[rs2 as usize] as u32),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_sllw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as u32) << (state.x[rs2 as usize] & 0x1f),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_srlw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as u32) >> (state.x[rs2 as usize] & 0x1f),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_sraw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    r!(insn, rd, rs1, rs2 => {
        let value = sign_extend!(
            (state.x[rs1 as usize] as i32) >> (state.x[rs2 as usize] & 0x1f),
            32
        );
        state.x[rd as usize] = value as u64;
        Ok(())
    })
}

pub fn rv64i_ecall(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    state.break_on = Some(BreakCause::Ecall);
    Ok(())
}

pub fn rv64i_ebreak(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    state.break_on = Some(BreakCause::Ebreak);
    Ok(())
}

pub fn rv64i_fence(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    Ok(())
}

pub fn rv64i_fence_i(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    Ok(())
}