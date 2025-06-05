
#[derive(Debug)]
pub struct Mmu {
    // Entry point of the loaded program
    pub(crate) entry: u64,
}