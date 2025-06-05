
use crate::*;

pub use super::mmu::*;
pub use super::state::*;

#[derive(Debug)]
pub struct Cpu {
    pub mmu: Mmu,
    pub state: State,
}

impl Cpu {
    pub fn load_program(&mut self, program: &[u8]) -> Result<Error> {
        todo!()
    }
}