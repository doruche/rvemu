
use crate::*;

pub use crate::guest::*;
pub use super::state::*;

#[derive(Debug)]
pub struct Machine {
    pub guest: GuestMem,
    pub state: State,
}

impl Machine {
    pub fn new() -> Self {
        todo!()
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result<Error> {
        todo!()
    }
}