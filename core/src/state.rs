//! Current state of the CPU, including registers and flags.


#[derive(Debug, Default)]
pub struct State {
    pub pc: u64,
    pub x: [u64; 32],
}

impl State {
    pub const ZERO: Self = Self {
        pc: 0,
        x: [0; 32],
    };
}