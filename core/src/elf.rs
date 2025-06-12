//! ELF64 support

use crate::*;

/// Size of the ELF identification array.
pub const EI_NIDENT: usize = 16;
/// Magic number for ELF files.
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
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

impl ElfHeader {
    pub fn from_bytes(src: &[u8]) -> Result<Self> {
        if src.len() != size_of::<Self>() {
            warn!("ELF header size mismatch: expected {}, got {}", size_of::<Self>(), src.len());
            return Err(Error::InvalidElf);
        }
        let res = unsafe {
            let mut hdr: Self = std::mem::zeroed();
            let src_ptr = src.as_ptr() as *const u8;
            std::ptr::copy_nonoverlapping(src_ptr, &mut hdr as *mut Self as *mut u8, size_of::<Self>());
            hdr
        };
        
        let magic = &res.e_ident[..4];
        if magic != ELF_MAGIC {
            warn!("Invalid ELF magic number: expected {:?}, got {:?}", ELF_MAGIC, magic);
            return Err(Error::InvalidElf);
        }

        if res.e_ident[EI_CLASS] != ELF_CLASS_64 {
            warn!("Unsupported ELF class: expected {}, got {}", ELF_CLASS_64, res.e_ident[EI_CLASS]);
            return Err(Error::InvalidElf);
        }

        if res.e_machine != EM_RISCV {
            warn!("Unsupported machine type: expected {}, got {}", EM_RISCV, res.e_machine);
            return Err(Error::InvalidElf);
        }

        Ok(res)
    }
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

impl ProgramHeader {
    pub fn from_bytes(src: &[u8]) -> Result<Self> {
        if src.len() != size_of::<Self>() {
            warn!("Program header size mismatch: expected {}, got {}", size_of::<Self>(), src.len());
            return Err(Error::InvalidElf);
        }
        let res = unsafe {
            let mut phdr: Self = std::mem::zeroed();
            let src_ptr = src.as_ptr() as *const u8;
            std::ptr::copy_nonoverlapping(src_ptr, &mut phdr as *mut Self as *mut u8, size_of::<Self>());
            phdr
        };
        Ok(res)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_header_from_bytes() {
        log::log_init(log::Level::Off);

        let mut header = [0u8; size_of::<ElfHeader>()];
        header[0..4].copy_from_slice(&ELF_MAGIC);
        header[EI_CLASS] = ELF_CLASS_64;

        let e_machine_offset = 0 + EI_NIDENT + 2;                
        header[e_machine_offset..e_machine_offset + 2].copy_from_slice(&EM_RISCV.to_le_bytes());

        let elf_header = ElfHeader::from_bytes(&header).unwrap();
        debug!("Parsed ELF header: {:?}", elf_header);
    }

    #[test]
    fn test_parse_file() {
        log::log_init(log::Level::Off);

        let elf = include_bytes!("../../testprogs/prime");
        let ehdr = ElfHeader::from_bytes(&elf[..size_of::<ElfHeader>()]).unwrap();
        debug!("{:#x?}", ehdr);

        for i in 0..ehdr.e_phnum as usize {
            let offset = ehdr.e_phoff as usize + i * ehdr.e_phentsize as usize;
            if offset + size_of::<ProgramHeader>() > elf.len() {
                warn!("Program header {} out of bounds", i);
                continue;
            }
            let phdr = ProgramHeader::from_bytes(&elf[offset..offset + size_of::<ProgramHeader>()]).unwrap();
            debug!("{:#x?}", phdr);
        }
    }
}