//! Dummy implementation of Zicsr, only for testing purposes.
//! Should not be used by users.
use crate::*;
use crate::error::*;
use crate::guest::*;
use crate::state::State;
use crate::insn::*;

pub const ZICSR_OPCODE: u8 = 0b1110011;
pub const ZICSR_FUNCT3_CSRRW: u8 = 0b001;
pub const ZICSR_FUNCT3_CSRRS: u8 = 0b010;
pub const ZICSR_FUNCT3_CSRRC: u8 = 0b011;
pub const ZICSR_FUNCT3_CSRRWI: u8 = 0b101;
pub const ZICSR_FUNCT3_CSRRSI: u8 = 0b110;
pub const ZICSR_FUNCT3_CSRRCI: u8 = 0b111;

pub const CSR_MHARTID: u32 = 0xF14;
pub const CSR_MEPC: u32 = 0x341;

#[derive(Debug)]
pub struct ZicsrDecoder;

pub static mut MEPC: u64 = 0;

impl Decoder for ZicsrDecoder {
    fn decode(&self, raw: u32) -> Result<Option<(Instruction, Executor)>> {
        let imm_i = Instruction::extract_imm(raw, InsnType::I);
        let opcode = (raw & 0x7f) as u8;
        let funct3 = ((raw >> 12) & 0x7) as u8;
        if opcode != ZICSR_OPCODE {
            return Ok(None);
        }
        let rd = ((raw >> 7) & 0x1f) as u8;
        let rs1 = ((raw >> 15) & 0x1f) as u8;
        let insn = Instruction::I {
            imm: imm_i,
            rd,
            rs1,
            funct3,
            opcode,
            raw,
        };

        let res = match funct3 {
            ZICSR_FUNCT3_CSRRW => (insn, zicsr_csrrw as Executor),
            ZICSR_FUNCT3_CSRRS => (insn, zicsr_csrrs as Executor),
            ZICSR_FUNCT3_CSRRC => (insn, zicsr_csrrc as Executor),
            ZICSR_FUNCT3_CSRRWI => (insn, zicsr_csrrwi as Executor),
            ZICSR_FUNCT3_CSRRSI => (insn, zicsr_csrrsi as Executor),
            ZICSR_FUNCT3_CSRRCI => (insn, zicsr_csrrci as Executor),
            0 => {
                match (rs1, rd, imm_i) {
                    (0, 0, 770) => (insn, priv_mret as Executor),
                    _ => {
                        debug!("Unsupported Zicsr instruction: funct3={:#x}, rd={}, rs1={}, imm={:#x}", funct3, rd, rs1, imm_i);
                        return Ok(None);
                    }
                }
            }
            _ => return Ok(None),
        };

        Ok(Some(res))
    }
}

fn zicsr_csrrw(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, imm, rs1, rd => {
        let csr = imm;
        match csr {
            CSR_MHARTID => {
                state.x[rd as usize] = 0;
            },
            CSR_MEPC => {
                unsafe {
                    state.x[rd as usize] = MEPC;
                    MEPC = state.x[rs1 as usize];
                }
            },
            _ => {
                debug!("Unsupported CSR operation: CSR={:#x}, rd={}, rs1={}", csr, rd, rs1);
            }
        }
        Ok(())
    })
}

fn zicsr_csrrs(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, imm, rs1, rd => {
        let csr = imm;
        match csr {
            CSR_MHARTID => {
                state.x[rd as usize] = 0;
            },
            CSR_MEPC => {
                unsafe {
                    state.x[rd as usize] = MEPC;
                    MEPC |= state.x[rs1 as usize];
                }
            },
            _ => {
                debug!("Unsupported CSR operation: CSR={:#x}, rd={}, rs1={}", csr, rd, rs1);
            }
        }
        Ok(())
    })
}

fn zicsr_csrrc(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, imm, rs1, rd => {
        let csr = imm;
        match csr {
            CSR_MHARTID => {
                state.x[rd as usize] = 0;
            },
            CSR_MEPC => {
                unsafe {
                    state.x[rd as usize] = MEPC;
                    MEPC &= !state.x[rs1 as usize];
                }
            },
            _ => {
                debug!("Unsupported CSR operation: CSR={:#x}, rd={}, rs1={}", csr, rd, rs1);
            }
        }
        Ok(())
    })
}

fn zicsr_csrrwi(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, imm, rs1, rd => {
        let csr = imm;
        let uimm = zero_extend!(rs1, 5);
        match csr {
            CSR_MHARTID => {
                state.x[rd as usize] = 0;
            },
            CSR_MEPC => {
                unsafe {
                    state.x[rd as usize] = MEPC;
                    MEPC = uimm;
                }
            },
            _ => {
                debug!("Unsupported CSR operation: CSR={:#x}, rd={}, rs1={}", csr, rd, rs1);
            }
        }
        Ok(())
    })
}

fn zicsr_csrrsi(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, imm, rs1, rd => {
        let csr = imm;
        let uimm = zero_extend!(rs1, 5);
        match csr {
            CSR_MHARTID => {
                state.x[rd as usize] = 0;
            },
            CSR_MEPC => {
                unsafe {
                    state.x[rd as usize] = MEPC;
                    MEPC |= uimm;
                }
            },
            _ => {
                debug!("Unsupported CSR operation: CSR={:#x}, rd={}, rs1={}", csr, rd, rs1);
            }
        }
        Ok(())
    })
}

fn zicsr_csrrci(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    i!(insn, imm, rs1, rd => {
        let csr = imm;
        let uimm = zero_extend!(rs1, 5);
        match csr {
            CSR_MHARTID => {
                state.x[rd as usize] = 0;
            },
            CSR_MEPC => {
                unsafe {
                    state.x[rd as usize] = MEPC;
                    MEPC &= !uimm;
                }
            },
            _ => {
                debug!("Unsupported CSR operation: CSR={:#x}, rd={}, rs1={}", csr, rd, rs1);
            }
        }
        Ok(())
    })
}

// These instructions should not appear here, but we do for convenience.
fn priv_mret(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    unsafe {
        state.pc = MEPC;
    }
    Ok(())
}