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
            trace!("loaded segment {:#x?}", segment);
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

        if (m_gaddr_start != gaddr_start) {
            warn!("Guest address {:#x} is not page-aligned, rounding down to {:#x}", gaddr_start, m_gaddr_start);
        }

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
            segment.host_mmap[(gaddr_start - m_gaddr_start) as usize + data.len()..].fill(0);
        }
        self.segments.insert(m_gaddr_start, segment);

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

    pub fn decompose_mut(&mut self, gaddr: u64, access: MemAccess) -> Result<(u64, &mut MemSegment)> {
        for (&base_gaddr, segment) in self.segments.range_mut(..=gaddr).rev() {
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

    pub fn read_u8(&self, gaddr: u64) -> Result<u8> {
        let (base_gaddr, segment) = self.decompose(gaddr, MemAccess::Read)?;
        let offset = (gaddr - segment.m_gaddr_start) as usize;
        Ok(segment.host_mmap[offset])
    }

    pub fn write_u8(&mut self, gaddr: u64, value: u8) -> Result<()> {
        let (base_gaddr, segment) = self.decompose_mut(gaddr, MemAccess::Write)?;
        let offset = (gaddr - segment.m_gaddr_start) as usize;
        segment.host_mmap[offset] = value;
        Ok(())
    }

    pub fn read_u16(&self, gaddr: u64) -> Result<u16> {
        // We can't ensure the address is aligned, so we read byte by byte.
        let low = self.read_u8(gaddr)?;
        let high = self.read_u8(gaddr + 1)?;
        Ok((high as u16) << 8 | (low as u16))
    }

    pub fn write_u16(&mut self, gaddr: u64, value: u16) -> Result<()> {
        self.write_u8(gaddr, (value & 0xFF) as u8)?;
        self.write_u8(gaddr + 1, (value >> 8) as u8)?;
        Ok(())
    }

    pub fn read_u32(&self, gaddr: u64) -> Result<u32> {
        let b0 = self.read_u8(gaddr)?;
        let b1 = self.read_u8(gaddr + 1)?;
        let b2 = self.read_u8(gaddr + 2)?;
        let b3 = self.read_u8(gaddr + 3)?;
        Ok((b3 as u32) << 24 | (b2 as u32) << 16 | (b1 as u32) << 8 | (b0 as u32))
    }

    pub fn write_u32(&mut self, gaddr: u64, value: u32) -> Result<()> {
        self.write_u8(gaddr, (value & 0xFF) as u8)?;
        self.write_u8(gaddr + 1, ((value >> 8) & 0xFF) as u8)?;
        self.write_u8(gaddr + 2, ((value >> 16) & 0xFF) as u8)?;
        self.write_u8(gaddr + 3, ((value >> 24) & 0xFF) as u8)?;
        Ok(())
    }

    pub fn read_u64(&self, gaddr: u64) -> Result<u64> {
        let mut res = [0u8; 8];
        for i in 0..8 {
            res[i] = self.read_u8(gaddr + i as u64)?;
        }
        Ok(u64::from_le_bytes(res))
    }

    pub fn write_u64(&mut self, gaddr: u64, value: u64) -> Result<()> {
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            self.write_u8(gaddr + i as u64, bytes[i])?;
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_elf() {
        log::log_init(log::Level::Off);

        let elf_data = include_bytes!("../../testprogs/prime");
        let mut guest_mem = GuestMem::new();
        let entry = guest_mem.load_elf(elf_data).expect("Failed to load ELF");
        debug!("ELF entry point: {:#x}", entry);
        debug!("Initial break address: {:#x}", guest_mem.init_brk_gaddr);
    }

    #[test]
    fn test_rw_bytes() {
        log::log_init(log::Level::Off);

        let elf_data = include_bytes!("../../testprogs/prime");
        let mut guest_mem = GuestMem::new();
        let entry = guest_mem.load_elf(elf_data).expect("Failed to load ELF");
        debug!("ELF entry point: {:#x}", entry);
        debug!("Initial break address: {:#x}", guest_mem.init_brk_gaddr);
        let test_addr = 0x1a000;
        guest_mem.write_u32(test_addr, 0x12345678).expect("Failed to write u32");
        let value = guest_mem.read_u32(test_addr).expect("Failed to read u32");
        assert_eq!(value, 0x12345678, "Read value does not match written value");
        guest_mem.write_u64(test_addr + 4, 0x9abcdef012345678).expect("Failed to write u64");
        let value64 = guest_mem.read_u64(test_addr + 4).expect("Failed to read u64");
        assert_eq!(value64, 0x9abcdef012345678, "Read value does not match written value");
        debug!("Read u32: {:#x}, Read u64: {:#x}", value, value64);
    }
}