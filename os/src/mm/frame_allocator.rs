use core::{error::Error, fmt::Display};

use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::debug;

use crate::{helper::cell::SingleThreadSafeCell, mm::address::{PhysAddr, PhysPageNumber, PAGE_SIZE_BYTES, PAGE_SIZE_WIDTH}};

lazy_static!{
    pub static ref FRAME_ALLOCATOR: SingleThreadSafeCell<StackFrameAllocator> = SingleThreadSafeCell::new(StackFrameAllocator::new());
}
pub trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNumber>;
    fn dealloc(&mut self, ppn: PhysPageNumber);
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator { current: 0, end: 0, recycled: Vec::new() }
    }
    
    fn alloc(&mut self) -> Option<PhysPageNumber> {
        if let Some(ppn) = self.recycled.pop() {
            return Some(ppn.into());
        }else{
            if self.current < self.end {
                let ppn = PhysPageNumber(self.current);
                self.current += 1;
                return Some(ppn);
            } else {
                return None; // No more pages available
            }
        }
    }
    
    fn dealloc(&mut self, ppn: PhysPageNumber) {
        let ppn: usize = ppn.into();
        if ppn > self.current || ppn >= self.end || self.recycled.contains(&ppn) {
            panic!("[MM] Invalid deallocation of page number: {}", ppn);
        }
        self.recycled.push(ppn);
    }

}

impl StackFrameAllocator {
    fn init(&mut self, start: PhysPageNumber, end: PhysPageNumber) {
        self.current = start.into();
        self.end = end.into();
    }
}


pub fn init_frame_allocator(_start: PhysAddr, end: PhysAddr) {
    unsafe extern "C" {
        fn ekernel();
    }
    debug!("[MM] Initializing frame allocator: [{:#x}, {:#x})", ekernel as usize, Into::<usize>::into(end));
    FRAME_ALLOCATOR.exclusive_access().init(Into::<PhysAddr>::into(ekernel as usize).ppn(), end.ppn());
}
#[derive(Debug)]
pub enum AllocErrorType {
    OutOfMemory
}
#[derive(Debug)]
pub struct AllocError{
    pub error_type: AllocErrorType,   
}
impl Display for AllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.error_type {
            AllocErrorType::OutOfMemory => write!(f, "Out of memory"),
        }
    }
}
impl Error for AllocError {
    
}
pub struct Frame{
    ppn: PhysPageNumber
}
impl Frame {
    pub fn new(ppn: PhysPageNumber) -> Self {
        unsafe{
            core::slice::from_raw_parts_mut(Into::<usize>::into(ppn.start_addr()) as *mut u8, PAGE_SIZE_BYTES).fill(0);
        }
        Frame { ppn }
    }
    pub fn ppn(&self) -> PhysPageNumber {
        self.ppn
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        FRAME_ALLOCATOR.exclusive_access().dealloc(self.ppn);
    }
}