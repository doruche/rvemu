//! Instruction decoding.

use std::fmt::Debug;

use crate::guest::GuestMem;
use crate::state::State;
use crate::*;
use crate::error::*;

#[derive(Debug)]
pub enum Instruction {
    R {
        // [31:25] funct7
        // [24:20] rs2
        // [19:15] rs1
        // [14:12] funct3
        // [11:7] rd
        // [6:0] opcode
        funct7: u8,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        rd: u8,
        opcode: u8,
        raw: u32,
    },
    I {
        // [31:20] imm[11:0]
        // [19:15] rs1
        // [14:12] funct3
        // [11:7] rd
        // [6:0] opcode
        imm: u16,
        rs1: u8,
        funct3: u8,
        rd: u8,
        opcode: u8,
        raw: u32,
    },
    S {
        // [31:25] imm[11:5]
        // [24:20] rs2
        // [19:15] rs1
        // [14:12] funct3
        // [11:7] imm[4:0]
        // [6:0] opcode
        imm: u16,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        opcode: u8,
        raw: u32,
    },
    B {
        // [31:25] imm[12, 10:5]
        // [24:20] rs2
        // [19:15] rs1
        // [14:12] funct3
        // [11:7] imm[4:1, 11]
        // [6:0] opcode
        imm: u16,
        rs2: u8,
        rs1: u8,
        funct3: u8,
        opcode: u8,
        raw: u32,
    },
    U {
        // [31:12] imm[31:12]
        // [11:7] rd
        // [6:0] opcode
        imm: u32,
        rd: u8,
        opcode: u8,
        raw: u32,
    },
    J {
        // [31:12] imm[20, 10:1, 11, 19:12]
        // [11:7] rd
        // [6:0] opcode
        imm: u32,
        rd: u8,
        opcode: u8,
        raw: u32,
    },

    R4 {
        // [31:27] fs3
        // [26:25] funct2
        // [24:20] fs2
        // [19:15] fs1
        // [14:12] funct3
        // [11:7] fd
        // [6:0] opcode
        fs3: u8,
        funct2: u8,
        fs2: u8,
        fs1: u8,
        funct3: u8,
        fd: u8,
        opcode: u8,
        raw: u32,
    },
    C {
        // TODO
        opcode: u8,
        raw: u32,
    }
}

impl Instruction {
    pub fn opcode(&self) -> u8 {
        match self {
            Instruction::R { opcode, .. } => *opcode,
            Instruction::I { opcode, .. } => *opcode,
            Instruction::S { opcode, .. } => *opcode,
            Instruction::B { opcode, .. } => *opcode,
            Instruction::U { opcode, .. } => *opcode,
            Instruction::J { opcode, .. } => *opcode,
            Instruction::R4 { opcode, .. } => *opcode,
            Instruction::C { opcode, .. } => *opcode,
        }
    }

    pub fn imm(&self) -> Option<u32> {
        use Instruction::*;
        match self {
            R {..} => None,
            I { imm, .. } => Some(*imm as u32),
            S { imm, .. } => Some(*imm as u32),
            B { imm, .. } => Some(*imm as u32),
            U { imm, .. } => Some(*imm),
            J { imm, .. } => Some(*imm),
            R4 { .. } => None,
            C { .. } => None,
        }
    }

    pub fn step_size(&self) -> usize {
        if let Instruction::C { .. } = self {
            2
        } else {
            4
        }
    }
}

macro_rules! u {
    ($insn:expr, $($field:ident),* => $body:block) => {
        if let &$crate::insn::Instruction::U { $($field),*, .. } = $insn {
            $body
        } else {
            return Err($crate::error::Error::InternalError(
                format!("Internal decoding error for {}", stringify!($insn))));
        } 
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsnSet {
    I32,
    I64,
    M,
    F,
    D,
    A,
    C,
}

pub trait Decoder: Debug {
    fn decode(&self, insn_raw: u32) -> Result<Option<(Instruction, Executor)>>;
}

pub type Executor = fn(&mut State, &mut GuestMem, &Instruction) -> Result<()>;

pub fn unimplemented_insn(state: &mut State, guest: &mut GuestMem, insn: &Instruction) -> Result<()> {
    error!("Unimplemented instruction: {:?}", insn);
    Err(Error::Unimplemented)
}



pub mod rv32i;

pub use rv32i::Rv32IDecoder;
