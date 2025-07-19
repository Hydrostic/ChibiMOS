pub(crate) mod heap_allocator;
pub(crate) mod address;
pub(crate) mod page_table;
mod frame_allocator;
pub(crate) mod memory_structure;

use lazy_static::lazy_static;
use log::{debug, info};

use crate::{helper::cell::SingleThreadSafeCell, io::dtb::DEVICE_TREE, mm::{frame_allocator::init_frame_allocator, memory_structure::{new_kernel_memory_set, MemorySet}}};

lazy_static!{
    pub static ref KERNEL_MEMORY_SET: SingleThreadSafeCell<Option<MemorySet>> = SingleThreadSafeCell::new(None);
}
pub fn init() {
    let dtb = DEVICE_TREE.exclusive_access();
    let dtb = dtb.as_ref().unwrap();
    if dtb.memory.is_empty() {
        panic!("[MM] No memory regions found in device tree");
    }
    debug!("[MM] Find {} memory regions", dtb.memory.len());
    debug!("[MM] Use first memory region: [{:#x}, {:#x})", dtb.memory[0].start, dtb.memory[0].end);
    init_frame_allocator(dtb.memory[0].start.into(), dtb.memory[0].end.into());
    let mut kms_ref = KERNEL_MEMORY_SET.exclusive_access();
    kms_ref.replace(new_kernel_memory_set(dtb.memory[0].start.into(), dtb.memory[0].end.into()));
    kms_ref.as_ref().unwrap().activate();
    debug!("[MM] Virtual memory enabled");
}