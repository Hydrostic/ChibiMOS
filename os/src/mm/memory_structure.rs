use core::cmp::min;
use core::{arch::asm, cmp::max, ops::Range};

use alloc::{collections::btree_map::BTreeMap, format, vec::Vec};
use elf::endian::AnyEndian;
use log::debug;
use riscv::register::satp::{self, Satp};
use riscv::register::satp::Mode as SatpMode;
use thiserror::Error;
use elf::{abi, ElfBytes, ParseError as ElfParseError};
use crate::mm::address::PhysAddr;
use crate::mm::{address::{IntoUsizeRange, PhysPageNumber, VirtAddr, VirtPageNumber, PAGE_SIZE_BYTES, PAGE_SIZE_WIDTH, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE}, frame_allocator::{Frame, FrameAllocator, FRAME_ALLOCATOR}, page_table::{PTEFlags, PageTable, PageTableError}};
use crate::sbi::putstr_debug;

pub enum MemoryAreaType {
    Identical,
    Framed
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct MemoryAreaPermissions: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4; // User mode access
    }
}
pub struct MemoryArea{
    vpn_range: Range<VirtPageNumber>,
    map_type: MemoryAreaType,
    map_permissions: MemoryAreaPermissions,
    frames: BTreeMap<VirtPageNumber, Frame>
}
#[derive(Debug, Error)]
pub enum MemoryStructureError {
    #[error(transparent)]
    PageTableEroor(#[from] PageTableError),
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Memory areas overlap")]
    OverlappedMemoryArea,
    #[error("Invalid memory area")]
    InvalidMemoryArea(Range<VirtPageNumber>),
    #[error("bad elf: {0}")]
    BadELF(#[from] ElfParseError),
    #[error("data too long")]
    DataTooLong,
    #[error("address not aligned")]
    AddressNotAligned,
    #[error("cross memory area not allowed")]
    CrossMemoryAreaNotAllowed
}

impl MemoryArea {
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, map_type: MemoryAreaType, map_permissions: MemoryAreaPermissions) -> Result<Self, MemoryStructureError> {
        // if Into::<usize>::into(start_va) & (1 << PAGE_SIZE_WIDTH) - 1 != 0
        // || Into::<usize>::into(end_va) & (1 << PAGE_SIZE_WIDTH) - 1 != 0 {
        //     return Err(MemoryStructureError::AddressNotAligned);
        // }
        let start_ppn = start_va.vpn();
        if Into::<usize>::into(end_va) == 0usize {
            return Err(MemoryStructureError::InvalidMemoryArea(start_ppn..start_ppn));
        }
        let end_ppn = VirtAddr(Into::<usize>::into(end_va) - 1usize).next_vpn();
        // let end_ppn = end_va.vpn();
        if start_ppn >= end_ppn {
            return Err(MemoryStructureError::InvalidMemoryArea(start_ppn..end_ppn));
        }
        Ok(MemoryArea {
            vpn_range: start_ppn..end_ppn,
            map_type,
            map_permissions,
            frames: BTreeMap::new()
        })
    }
    pub fn map(&mut self, page_table: &mut PageTable) -> Result<(), MemoryStructureError> {
        for vpn in self.vpn_range.clone().into_usize_range(){
            self.map_one(page_table, vpn.into())?;
        }
        Ok(())
    }
    pub fn unmap(&self, page_table: &mut PageTable) -> Result<(), MemoryStructureError> {
        for vpn in self.vpn_range.clone().into_usize_range(){
            self.unmap_one(page_table, vpn.into())?;
        }
        Ok(())
    }

    fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNumber) -> Result<(), MemoryStructureError> {
        let ppn: PhysPageNumber;
        match self.map_type {
            MemoryAreaType::Identical => {
                ppn = Into::<usize>::into(vpn).into();
            },
            MemoryAreaType::Framed => {
                ppn = FRAME_ALLOCATOR.exclusive_access().alloc().ok_or(
                    MemoryStructureError::OutOfMemory)?;
                self.frames.insert(vpn, Frame::new(ppn));
            }
        }
        let flags = PTEFlags::from_bits(self.map_permissions.bits()).unwrap();
        page_table.map(vpn, ppn, flags)?;
        Ok(())
    }

    fn unmap_one(&self, page_table: &mut PageTable, vpn: VirtPageNumber) -> Result<(), MemoryStructureError>{
        if let MemoryAreaType::Framed = self.map_type {
                let frame = self.frames.get(&vpn).unwrap();
                FRAME_ALLOCATOR.exclusive_access().dealloc(frame.ppn());
        }
        page_table.unmap(vpn)?;
        Ok(())
    }

    pub fn has_overlap_with(&self, other: &MemoryArea) -> bool {
        let t = self.vpn_range.start <= other.vpn_range.end && other.vpn_range.start < self.vpn_range.end;
        if t {
            log::debug!("[MM] Memory area overlap detected: [{}, {}) overlaps with [{}, {})", 
                self.vpn_range.start, self.vpn_range.end, 
                other.vpn_range.start, other.vpn_range.end);
        }
        t
    }

}

#[allow(dead_code)]
unsafe extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}
pub struct AddressIterator<'a>{
    start_va: VirtAddr,
    end_va: VirtAddr,
    page_table: &'a PageTable
}
impl<'a> AddressIterator<'a>{
    pub fn new(start_va: VirtAddr, end_va: VirtAddr, page_table: &'a PageTable) -> Self {
        Self {
            start_va,
            end_va,
            page_table
        }
    }
}

impl<'a> Iterator for AddressIterator<'a> {
    type Item = (PhysAddr, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start_va >= self.end_va {
            return None;
        }
        let vpn = self.start_va.vpn();
        let offset = self.start_va - vpn.start_addr();
        let ppn = self.page_table.translate(vpn).ok()?.ppn();
        let start_pa = ppn.start_addr() + Into::<usize>::into(offset);
        let current_page_end_va = vpn.end_addr();
        let old_start_va = self.start_va;
        if self.end_va >= current_page_end_va{
            self.start_va = current_page_end_va;
            Some((start_pa, (current_page_end_va - old_start_va).into()))
        }else{
            self.start_va = self.end_va;
            Some((start_pa, (self.end_va - old_start_va).into()))
        }
    }
}
impl<'a> ExactSizeIterator for AddressIterator<'a> {
    fn len(&self) -> usize {
        Into::<usize>::into(self.end_va.vpn().end_addr() - self.start_va.vpn().start_addr()) >> PAGE_SIZE_WIDTH
    }
}
pub struct MemorySet{
    page_table: PageTable,
    areas: Vec<MemoryArea>
}
impl MemorySet {
    pub fn new() -> Result<Self, MemoryStructureError>{
        Ok(MemorySet { page_table: PageTable::new()?, areas: Vec::new() })
    }

    pub fn push(&mut self, mut area: MemoryArea, data: Option<&[u8]>) -> Result<(), MemoryStructureError> {
        if self.areas.iter().any(|a| a.has_overlap_with(&area)) {
            return Err(MemoryStructureError::OverlappedMemoryArea);
        }
        area.map(&mut self.page_table)?;
        if let Some(data) = data {
            let data_pages = (data.len() + PAGE_SIZE_BYTES - 1) / (PAGE_SIZE_BYTES);
            if data_pages > area.frames.len() {
                return Err(MemoryStructureError::DataTooLong);
            }
            for i in 0..data_pages {
                let vpn = area.vpn_range.start + i;
                let frame = area.frames.get_mut(&vpn).unwrap();
                let data_slice = &data[i * PAGE_SIZE_BYTES..min((i + 1) * PAGE_SIZE_BYTES, data.len())]; // 错误，，复制区域计算错误
                frame.ppn().get_mut_array::<u8>()[0..data_slice.len()].copy_from_slice(data_slice);
            }
        }
        self.areas.push(area);
        Ok(())
    }

    pub fn translate(&self, va: VirtAddr) -> Result<PhysPageNumber, MemoryStructureError> {
        let vpn = va.vpn();
        Ok(self.page_table.translate(vpn)?.ppn())
    }

    pub fn translate_byte_buffer(&self, va: VirtAddr, len: usize) -> Result<AddressIterator, MemoryStructureError> {
        let vpn = va.vpn();
        for area in self.areas.iter() {
            if area.vpn_range.contains(&vpn){
                return if va + len > area.vpn_range.end.start_addr() {
                    Err(MemoryStructureError::CrossMemoryAreaNotAllowed)
                } else {
                    Ok(AddressIterator::new(va, va + len, &self.page_table))
                }
            }
        }
        Err(MemoryStructureError::PageTableEroor(PageTableError::NoMapExists(vpn)))
    }
    pub fn map_trampoline(&mut self) -> Result<(), MemoryStructureError> {
        log::debug!("[MM] Mapping trampoline at {:#x} -> {:#x}", TRAMPOLINE, strampoline as usize);
        self.page_table.map(VirtAddr(TRAMPOLINE).vpn(), 
            PhysAddr(strampoline as usize).ppn(), 
            PTEFlags::R | PTEFlags::X)?;
        Ok(())
    }

    pub fn activate(&self) {
        unsafe{
            satp::set(SatpMode::Sv39, 0, self.page_table.root_ppn());
            asm!("sfence.vma")
        }
    }

    pub fn token(&self) -> usize {
        // let mut s = Satp::from_bits(0);
        // s.set_mode(SatpMode::Sv39);
        // s.set_ppn(self.page_table.root_ppn());
        // s.bits()
        self.page_table.token()
    }
}


pub fn new_kernel_memory_set(start: usize, end: usize) -> MemorySet{
    let mut set = MemorySet::new().expect("[MM] Failed to create kernel memory set");

    set.map_trampoline().expect("Failed to map trampoline");
    debug!("[MM] Kernel segment .text [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!("[MM] Kernel segment .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    debug!("[MM] Kernel segment .data [{:#x}, {:#x})", sdata as usize, edata as usize);
    debug!("[MM] Kernel segment .bss(include stack) [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);

    set.push(MemoryArea::new((stext as usize).into(), (etext as usize).into(), 
        MemoryAreaType::Identical, 
        MemoryAreaPermissions::R | MemoryAreaPermissions::X).unwrap(), None).expect("[MM] Failed to push .text area");

    set.push(MemoryArea::new((srodata as usize).into(), (erodata as usize).into(), 
        MemoryAreaType::Identical, 
        MemoryAreaPermissions::R).unwrap(), None).expect("[MM] Failed to push .rodata area");
    
    set.push(MemoryArea::new((sdata as usize).into(), (edata as usize).into(), 
        MemoryAreaType::Identical, 
        MemoryAreaPermissions::R | MemoryAreaPermissions::W).unwrap(), None).expect("[MM] Failed to push .data area");
    
    set.push(MemoryArea::new((sbss_with_stack as usize).into(), (ebss as usize).into(), 
        MemoryAreaType::Identical, 
        MemoryAreaPermissions::R | MemoryAreaPermissions::W).unwrap(), None).expect("[MM] Failed to push .bss area");

    set.push(MemoryArea::new((ekernel as usize).into(), end.into(), 
        MemoryAreaType::Identical, 
        MemoryAreaPermissions::R | MemoryAreaPermissions::W).unwrap(), None).expect("[MM] Failed to push left area");
    set
}

pub fn new_elf_memory_set(process_index: usize, elf_raw: &[u8]) -> Result<(MemorySet, usize, usize), MemoryStructureError> {
    log::debug!("[MM] Initializing ELF memory set for process {}", process_index);
    let mut set = MemorySet::new()?;

    set.map_trampoline()?;
    // let elf = xmas_elf::ElfFile::new(elf).map_err(|e| MemoryStructureError::BadELF(e))?;
    let elf : ElfBytes<'_, AnyEndian> = ElfBytes::minimal_parse(elf_raw)?;
    let mut max_end_va: VirtAddr = 0.into();
    if let Some(iter) = elf.segments(){
        for header in iter {
            if header.p_type == abi::PT_LOAD {
                let start_va: VirtAddr = (header.p_vaddr as usize).into();
                let end_va: VirtAddr = ((header.p_vaddr + header.p_memsz) as usize).into();
                let mut area_permission = MemoryAreaPermissions::U;
                if header.p_flags & abi::PF_X != 0 { area_permission |= MemoryAreaPermissions::X; }
                if header.p_flags & abi::PF_R != 0 { area_permission |= MemoryAreaPermissions::R; }
                if header.p_flags & abi::PF_W != 0 { area_permission |= MemoryAreaPermissions::W; }
                let area = MemoryArea::new(start_va, end_va, 
                    MemoryAreaType::Framed, area_permission)?;
                log::debug!("[MM] User program segment: [{start_va}, {end_va})");
                max_end_va = max(max_end_va, end_va);
                let segment_img_start = header.p_offset as usize;
                let segment_img = &elf_raw[segment_img_start..segment_img_start + header.p_filesz as usize];
                set.push(area, Some(segment_img))?;

            }
        }
    }
    let user_stack_bottom = (max_end_va.vpn() + 1).start_addr();
    let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
    log::debug!("[MM] User stack segment: [{user_stack_bottom}, {user_stack_top})");
    let stack_area = MemoryArea::new(user_stack_bottom, user_stack_top,
        MemoryAreaType::Framed, 
        MemoryAreaPermissions::R | MemoryAreaPermissions::W | MemoryAreaPermissions::U)?;
    set.push(stack_area, None)?;
    log::debug!("[MM] User trap context segment: [{:#x}, {:#x})", TRAP_CONTEXT, TRAMPOLINE);
    let trap_context_area = MemoryArea::new(TRAP_CONTEXT.into(), TRAMPOLINE.into(),
        MemoryAreaType::Framed,
        MemoryAreaPermissions::R | MemoryAreaPermissions::W | MemoryAreaPermissions::X)?;

    set.push(trap_context_area, None)?;
    log::debug!("[MM] User program entry point: {:#x}", elf.ehdr.e_entry as usize);
    Ok((set, user_stack_top.into(), elf.ehdr.e_entry as usize))
}