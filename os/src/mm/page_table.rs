use core::cmp::{max, min};

use alloc::vec::Vec;
use riscv::register::satp::Satp;
use thiserror::Error;

use crate::mm::{address::{PhysPageNumber, VirtAddr, VirtPageNumber, PAGE_SIZE_BYTES}, frame_allocator::{Frame, FrameAllocator, FRAME_ALLOCATOR}};
#[derive(Debug, Error)]
pub enum PageTableError {
    #[error("Frame unavailable")]
    FrameUnavailable,
    #[error("Mapping already exists for VPN {0}")]
    MapAlreadyExists(VirtPageNumber),
    #[error("No mapping exists for VPN {0}")]
    NoMapExists(VirtPageNumber),
    #[error("Address overflow occurred during translation")]
    AddressOverflow
}
bitflags! {
    #[derive(Clone, Copy)]
    #[repr(C)]
    pub struct PTEFlags: u8 {

        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new(ppn: PhysPageNumber, flags: PTEFlags) -> Self {
        let mut entry = 0;
        entry |= ppn.0 << 10;
        entry |= flags.bits() as usize;
        PageTableEntry(entry)
    }
    
    pub fn ppn(&self) -> PhysPageNumber {
        (self.0 >> 10 & ((1usize << 44) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits((self.0 & 0xFF) as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn empty() -> Self {
        PageTableEntry(0)
    }

}

pub struct PageTable{
    root_ppn: PhysPageNumber,
    frames: Vec<Frame> // frames that storage the page table entries
}

impl PageTable {
    pub fn new() -> Result<Self, PageTableError> {
        let root_ppn = FRAME_ALLOCATOR.exclusive_access().alloc().ok_or(PageTableError::FrameUnavailable)?;
        let mut frames = Vec::new();
        frames.push(Frame::new(root_ppn));
        Ok(PageTable { root_ppn, frames })
    }
    fn find_pte_create(&mut self, vpn: VirtPageNumber) -> Result<&mut PageTableEntry, PageTableError> {
        let indexes = vpn.indexes();
        let mut current_ppn = self.root_ppn;
        for i in 0..3 {
            let entry = current_ppn.get_mut_array::<PageTableEntry>().get_mut(indexes[i]).unwrap();
            if i == 2 {
                return Ok(entry);
            }
            if !entry.is_valid() {
                let new_ppn = FRAME_ALLOCATOR.exclusive_access().alloc().ok_or(
                    PageTableError::FrameUnavailable)?;
                *entry = PageTableEntry::new(new_ppn, PTEFlags::V);
                self.frames.push(Frame::new(new_ppn));
            }
            current_ppn = entry.ppn();
        }
        unreachable!()
    }
    fn find_pte(&self, vpn: VirtPageNumber) -> Option<&mut PageTableEntry> {
        let indexes = vpn.indexes();
        let mut current_ppn = self.root_ppn;
        for i in 0..3 {
            let entry = current_ppn.get_mut_array::<PageTableEntry>().get_mut(indexes[i]).unwrap();
            if !entry.is_valid() {
                return None;
            }
            if i == 2 {
                return Some(entry);
            }
            current_ppn = entry.ppn();
        }
        unreachable!()
    }
    pub fn map(&mut self, vpn: VirtPageNumber, ppn: PhysPageNumber, flags: PTEFlags) -> Result<(), PageTableError> {
        let entry = self.find_pte_create(vpn)?;
        if entry.is_valid() {
            return Err(PageTableError::MapAlreadyExists(vpn));
        }
        *entry = PageTableEntry::new(ppn, flags | PTEFlags::V);
        Ok(())
    }

    pub fn unmap(&mut self, vpn: VirtPageNumber) -> Result<(), PageTableError> {
        let entry = self.find_pte(vpn).ok_or(PageTableError::NoMapExists(vpn))?;
        if !entry.is_valid() {
            return Err(PageTableError::NoMapExists(vpn));
        }
        *entry = PageTableEntry::empty();
        Ok(())
    }

    pub fn translate(&self, vpn: VirtPageNumber) -> Result<PageTableEntry, PageTableError> {
        let entry = self.find_pte(vpn).ok_or(PageTableError::NoMapExists(vpn))?;
        if !entry.is_valid() {
            return Err(PageTableError::NoMapExists(vpn));
        }
        Ok(entry.clone())
    }

    pub fn root_ppn(&self) -> usize {
        self.root_ppn.into()
    }


    pub fn from_token(token: usize) -> Self {
        Self {
            root_ppn: PhysPageNumber(token & ((1usize << 44) - 1)),
            frames: Vec::new()
        }
    }
    // TODO: 大小溢出
    pub fn translate_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Result<Vec<&'static [u8]>, PageTableError>{
        let page_table = PageTable::from_token(token);
        let start_addr = ptr as usize;
        let start_vpn = VirtAddr(start_addr).vpn();
        if (start_addr + len) == 0 {
            return Err(PageTableError::AddressOverflow);
        }
        let end_vpn = VirtAddr(start_addr + len - 1).next_vpn();
        let pages: usize = (end_vpn - start_vpn).into();
        let mut buffer_ref_array = Vec::<&'static [u8]>::new();
        let mut current_vpn = start_vpn;
        for _i in 0..pages {
            let vpn_start_addr = Into::<usize>::into(current_vpn.start_addr());
            let start = max(ptr as usize, vpn_start_addr) - vpn_start_addr;
            let end = min(Into::<usize>::into(current_vpn.end_addr()), ptr as usize + len) - vpn_start_addr;
            let ppn = page_table.translate(current_vpn)?.ppn();
            buffer_ref_array.push(&ppn.get_array::<u8>()[start..end]);
            current_vpn = current_vpn + 1;
        }

        Ok(buffer_ref_array)
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}