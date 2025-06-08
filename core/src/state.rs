//! Current state of the CPU, including registers and flags.


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BreakCause {
    DirectBranch,
    IndirectBranch,
    Ecall,
    Ebreak
}

#[derive(Debug, Default)]
pub struct State {
    pub pc: u64,
    pub x: [u64; 32],
    pub break_on: Option<BreakCause>,
}

impl State {
    pub const ZERO: Self = Self {
        pc: 0,
        x: [0; 32],
        break_on: None,
    };
}