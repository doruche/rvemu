//! Memory management for guest programs.

use std::collections::BTreeMap;
use bitflags::bitflags;
use memmap2::{MmapMut, MmapOptions};
use crate::*;
use crate::elf::*;

const PAGE_SIZE: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemAccess {
    Read,
    Write,
    Execute,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MemFlags: u8 {
        const NONE = 0;
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
    }
}

impl MemFlags {
    pub fn from_p_flags(p_flags: u32) -> Self {
        let mut flags = MemFlags::NONE;
        if p_flags & PF_R != 0 {
            flags.insert(MemFlags::READ);
        }
        if p_flags & PF_W != 0 {
            flags.insert(MemFlags::WRITE);
        }
        if p_flags & PF_X != 0 {
            flags.insert(MemFlags::EXECUTE);
        }
        flags
    }
}

#[derive(Debug)]
pub struct MemSegment {
    // [gaddr_start, gaddr_end)
    gaddr_start: u64,
    gaddr_end: u64,
    // 'm' means page-aligned.
    // for bss/sbss segments,it additionally indicates more memory pages.
    m_gaddr_start: u64,
    m_gaddr_end: u64,
    host_mmap: MmapMut,
    flags: MemFlags,
}

impl MemSegment {
    pub fn new(
        gaddr_start: u64, 
        gaddr_end: u64, 
        m_gaddr_start: u64,
        m_gaddr_end: u64,
        host_mmap: MmapMut, 
        flags: MemFlags
    ) -> Self {
        assert!(gaddr_start < gaddr_end, "Invalid memory segment range");
        Self {
            gaddr_start,
            gaddr_end,
            m_gaddr_start,
            m_gaddr_end,
            host_mmap,
            flags,
        }
    }

    pub fn num_pages(&self) -> usize {
        (self.m_gaddr_end - self.m_gaddr_start) as usize / PAGE_SIZE
    }

    pub fn host_ptr(&self) -> *const u8 {
        self.host_mmap.as_ptr()
    }

    pub fn host_ptr_mut(&mut self) -> *mut u8 {
        self.host_mmap.as_mut_ptr()
    }

    pub fn contains(&self, guest_addr: u64) -> bool {
        guest_addr >= self.gaddr_start && guest_addr < self.m_gaddr_end
    }

    pub fn allows(&self, access: MemAccess) -> bool {
        match access {
            MemAccess::Read => self.flags.contains(MemFlags::READ),
            MemAccess::Write => self.flags.contains(MemFlags::WRITE),
            MemAccess::Execute => self.flags.contains(MemFlags::EXECUTE),
        }
    }

    pub fn flags(&self) -> MemFlags {
        self.flags
    }
}

#[derive(Debug)]
pub struct GuestMem {
    /// (base address, segment)
    segments: BTreeMap<u64, MemSegment>,
    init_brk_gaddr: u64,
    cur_brk_gaddr: u64,
    stk_base_gaddr: u64,
    stk_size: usize,
}

impl GuestMem {
    pub fn new() -> Self {
        Self {
            segments: BTreeMap::new(),
            init_brk_gaddr: 0,
            cur_brk_gaddr: 0,
            stk_base_gaddr: 0,
            stk_size: 0,
        }
    }

    pub fn load_elf(&mut self, elf: &[u8]) -> Result<u64> {
        if elf.len() < size_of::<ElfHeader>() {
            warn!("ELF file too small: {} bytes", elf.len());
            return Err(Error::InvalidElfHdr);
        }
        let ehdr = ElfHeader::from_bytes(&elf[..size_of::<ElfHeader>()])?;
        let entry = ehdr.e_entry;

        // load program segments
        let mut phdr: ProgramHeader;
        for i in 0..ehdr.e_phnum as usize {
            let phdr_offset = ehdr.e_phoff as usize + (i * size_of::<ProgramHeader>());
            phdr = ProgramHeader::from_bytes(
                &elf[phdr_offset..phdr_offset + size_of::<ProgramHeader>()]
            )?;

            if phdr.p_type == PT_LOAD {
                let flags = MemFlags::from_p_flags(phdr.p_flags);
                let init_data = Some(&elf[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize]);
                self.add_segment(
                    phdr.p_vaddr,
                    phdr.p_memsz as usize,
                    flags,
                    init_data
                )?;
            }
        }

        let mut init_brk_gaddr = 0;
        for (&gaddr_start, segment) in self.segments.iter() {
            debug!("loaded segment {:#x?}", segment);
            init_brk_gaddr = (init_brk_gaddr).max(segment.m_gaddr_end);
        }
        self.init_brk_gaddr = init_brk_gaddr;
        self.cur_brk_gaddr = init_brk_gaddr;

        Ok(entry)
    }

    pub fn add_segment(
        &mut self,
        gaddr_start: u64,
        len: usize,
        flags: MemFlags,
        init_data: Option<&[u8]>,
    ) -> Result<()> {
        assert!(len != 0);

        let gaddr_end = gaddr_start + len as u64;

        let m_gaddr_start = round_down!(gaddr_start, PAGE_SIZE) as u64;
        let m_gaddr_end = round_up!(gaddr_end, PAGE_SIZE) as u64;
        let m_len = m_gaddr_end - m_gaddr_start as u64;

        // Check if the segment overlaps with existing segments
        for (&seg_base_gaddr, seg) in self.segments.iter() {
            if (m_gaddr_start < seg.gaddr_start && m_gaddr_end > seg.gaddr_start) ||
               (m_gaddr_start < seg.gaddr_end && m_gaddr_end > seg.gaddr_end) ||
               (m_gaddr_start >= seg.gaddr_start && m_gaddr_end <= seg.gaddr_end) ||
               (m_gaddr_start <= seg.gaddr_start && m_gaddr_end >= seg.gaddr_end) {
                warn!("Memory segment overlaps with existing segment at base address {:#x}", seg_base_gaddr);
                return Err(Error::SegmentOverlap);
            }
        }

        let mmap = MmapOptions::new()
            .len(m_len as usize)
            .map_anon()
            .map_err(|e| {
                warn!("Failed to create memory map: {}", e);
                Error::InternalError(format!("Failed to create memory map: {}", e))
            })?;
        
        let mut segment = MemSegment::new(
            gaddr_start, gaddr_end,
            m_gaddr_start, m_gaddr_end, 
            mmap, flags,
        );

        if let Some(data) = init_data {
            if data.len() > len {
                warn!("Initialization data exceeds requested size: {} > {}", data.len(), len);
                return Err(Error::OutOfBounds);
            }
            // copy init data
            segment.host_mmap[(gaddr_start - m_gaddr_start) as usize..(gaddr_start - m_gaddr_start) as usize + data.len()]
                .copy_from_slice(data);
            // zero out the rest of the segment
            segment.host_mmap[..(gaddr_start - m_gaddr_start) as usize].fill(0);
            segment.host_mmap[(m_gaddr_end - gaddr_end) as usize..].fill(0);
        }
        self.segments.insert(gaddr_start, segment);

        Ok(())
    }

    /// Decomposes a guest address into its segment and checks access permissions.
    fn decompose(&self, gaddr: u64, access: MemAccess) -> Result<(u64, &MemSegment)> {
        for (&base_gaddr, segment) in self.segments.range(..=gaddr).rev() {
            if segment.contains(gaddr) {
                if segment.allows(access) {
                    return Ok((base_gaddr, segment));
                } else {
                    warn!("Access denied for address {:#x} with flags {:?}", gaddr, access);
                    return Err(Error::PermissionDenied)
                }
            }
        }
        warn!("Address {:#x} not found in any memory segment", gaddr);
        Err(Error::MemAccessFault(access, gaddr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_elf() {
        log::log_init(log::Level::Off);

        let elf_data = include_bytes!("../../testfile/prime");
        let mut guest_mem = GuestMem::new();
        let entry = guest_mem.load_elf(elf_data).expect("Failed to load ELF");
        debug!("ELF entry point: {:#x}", entry);
        debug!("Initial break address: {:#x}", guest_mem.init_brk_gaddr);
    }
}