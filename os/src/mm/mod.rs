pub(crate) mod heap_allocator;
pub(crate) mod address;
pub(crate) mod page_table;
mod frame_allocator;
pub(crate) mod memory_structure;
pub(crate) mod kernel;

use lazy_static::lazy_static;
use log::{debug, info};

use crate::{helper::cell::SingleThreadSafeCell, io::dtb::DEVICE_TREE, mm::{frame_allocator::init_frame_allocator, memory_structure::MemorySet}};
use crate::mm::kernel::KERNEL_MEMORY_MANAGER;

// lazy_static!{
//     pub static ref KERNEL_MEMORY_SET: SingleThreadSafeCell<Option<MemorySet>> = SingleThreadSafeCell::new(None);
// }
pub fn init() {
    let dtb = DEVICE_TREE.exclusive_access();
    let dtb = dtb.as_ref().unwrap();
    if dtb.memory.is_empty() {
        panic!("[MM] No memory regions found in device tree");
    }
    debug!("[MM] Find {} memory regions", dtb.memory.len());
    let start = dtb.memory[0].start;
    let end = dtb.memory[0].end;
    debug!("[MM] Use first memory region: [{:#x}, {:#x})", start, end);
    init_frame_allocator(dtb.memory[0].start.into(), dtb.memory[0].end.into());
    KERNEL_MEMORY_MANAGER.exclusive_access().init(start, end);
    debug!("[MM] Virtual memory enabled");
}