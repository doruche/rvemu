//! Instruction decoding.

use std::fmt::Debug;

use crate::guest::GuestMem;
use crate::state::State;
use crate::*;
use crate::error::*;

/// The 'imm' field has not been sign-extended yet.
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
        imm: u32,
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
        imm: u32,
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
        imm: u32,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsnType {
    R,
    I,
    S,
    B,
    U,
    J,
    R4,
    C,
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

    pub fn extract_imm(raw: u32, insn_type: InsnType) -> u32 {
        use InsnType::*;
        match insn_type {
            I => raw >> 20,
            S => (((raw >> 25) & 0x7f) << 5) | ((raw >> 7) & 0x1f),
            B => (((raw >> 31) & 0x1) << 12) | (((raw >> 25) & 0x3f) << 5) | (((raw >> 8) & 0xf) << 1) | (((raw >> 7) & 0x1) << 11),
            U => raw & 0xfffff000,
            J => (((raw >> 31) & 0x1) << 20) | (((raw >> 21) & 0x3ff) << 1) | (((raw >> 20) & 0x1) << 11) | (((raw >> 12) & 0xff) << 12),
            _ => panic!("extract_imm called with unsupported instruction type: {:?}", insn_type),
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

macro_rules! gen_insn_unwrappers {
    ($dollar:tt, $($name:ident, $type:ident),*) => {
        $(
            macro_rules! $name {
                ($insn:expr, $dollar($field:ident),* => $body:block) => {
                    if let &$crate::insn::Instruction::$type { $dollar($field),*, .. } = $insn {
                        $body
                    } else {
                        return Err($crate::error::Error::InternalError(
                            format!("Internal decoding error for {}", stringify!($insn))));
                    }
                };
            }
        )*
    };
}

gen_insn_unwrappers!(
    $,
    r, R,
    i, I,
    s, S,
    b, B,
    u, U,
    j, J
);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsnSet {
    I,
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


pub mod rv64i;

pub use rv64i::Rv64IDecoder;


#[cfg(test)]
mod tests {
    use std::net::Incoming;

    use super::*;

    #[test]
    fn test_extract_imm() {
        log::log_init(log::Level::Off);

        // I-type
        let addi =0x02010113;
        let imm = Instruction::extract_imm(addi, InsnType::I);
        assert_eq!(imm, 0x20);

        let addi = 0x06400293;
        let imm = Instruction::extract_imm(addi, InsnType::I);
        assert_eq!(imm, 0x64);

        let addi = 0xfff00313;
        let imm = Instruction::extract_imm(addi, InsnType::I);
        assert_eq!(((imm as i32) << 20) >> 20, -1);

        let lw = 0x00842303;
        let imm = Instruction::extract_imm(lw, InsnType::I);
        assert_eq!(imm, 0x8);

        let lb = 0xFFC50483;
        let imm = Instruction::extract_imm(lb, InsnType::I);
        assert_eq!(((imm as i32) << 20) >> 20, -4);

        // S-type
        let sw = 0x00532623;
        let imm = Instruction::extract_imm(sw, InsnType::S);
        assert_eq!(imm, 12);

        let sb = 0xfe740c23;
        let imm = Instruction::extract_imm(sb, InsnType::S);
        assert_eq!(((imm as i32) << 20) >> 20, -8);

        // B-type
        let beq = 0x00000463;
        let imm = Instruction::extract_imm(beq, InsnType::B);
        assert_eq!(imm, 8);

        let bne = 0xffd11ee3;
        let imm = Instruction::extract_imm(bne, InsnType::B);
        assert_eq!(((imm as i32) << 19) >> 19, -4);

        // U-type
        let lui = 0x12345537;
        let imm = Instruction::extract_imm(lui, InsnType::U);
        assert_eq!(imm, 0x12345 << 12);

        let auipc = 0xfffff5bb;
        let imm = Instruction::extract_imm(auipc, InsnType::U);
        assert_eq!(imm, 0xfffff << 12);

        // J-type
        let jal = 0x028000ef;
        let imm = Instruction::extract_imm(jal, InsnType::J);
        assert_eq!(imm, 40);

        let jal = 0xff80006f;
        let imm = Instruction::extract_imm(jal, InsnType::J);
        assert_eq!(((imm as i32) << 11) >> 11, -1046536);
        assert_eq!(sign_extend!(imm, 21), -1046536);
    }
}