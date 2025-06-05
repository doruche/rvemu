//! ELF64 support

/// Size of the ELF identification array.
pub const EI_NIDENT: usize = 16;
/// Magic number for ELF files.
pub const ELF_MAGIX: [u8; 4] = [0x7f, b'E', b'L', b'F'];
/// RISC-V machine type.
pub const EM_RISCV: u16 = 0xf3;
/// Index of ELF class in e_ident.
pub const EI_CLASS: usize = 4;

/// Invalid or unknown ELF class.
pub const ELF_CLASS_NONE: u8 = 0;
/// 32-bit ELF class.
pub const ELF_CLASS_32: u8 = 1;
/// 64-bit ELF class.
pub const ELF_CLASS_64: u8 = 2;
/// Number of defined ELF classes.
pub const ELF_CLASS_NUM: u8 = 3;

/// Loadable segment type.
pub const PT_LOAD: u32 = 1;

pub const PF_X: u32 = 0x1;
pub const PF_W: u32 = 0x2;
pub const PF_R: u32 = 0x4;

/// PC-relative 32-bit relocation
pub const R_X86_64_PC32: u32 = 2;

#[repr(C)]
#[derive(Debug)]
pub struct ElfHeader {
    pub e_ident: [u8; EI_NIDENT],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct SectionHeader {
    pub sh_name: u32,
    pub sh_type: u32,
    pub sh_flags: u64,
    pub sh_addr: u64,
    pub sh_offset: u64,
    pub sh_size: u64,
    pub sh_link: u32,
    pub sh_info: u32,
    pub sh_addralign: u64,
    pub sh_entsize: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct Symbol {
    pub st_name: u32,
    pub st_info: u8,
    pub st_other: u8,
    pub st_shndx: u16,
    pub st_value: u64,
    pub st_size: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct Relocation {
    pub r_offset: u64,
    pub r_type: u32,
    pub r_sym: u32,
    pub r_addend: i64,
}