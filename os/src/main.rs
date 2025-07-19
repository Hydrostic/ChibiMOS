#![no_std]
#![no_main]
#![feature(sync_unsafe_cell)]
#![feature(alloc_error_handler)]
#![feature(fn_align)]
mod helper;
mod lang_items;
mod sbi;
#[macro_use]
mod console;
mod logging;
mod loader;
mod trap;
mod syscall;
mod task;
mod timer;
mod mm;
mod io;

use core::arch::global_asm;
use dtb_walker::{utils::indent, Dtb, DtbObj, WalkOperation, Property};
use log::{debug, error, info, warn};
extern crate alloc;
#[macro_use]
extern crate static_assertions;
#[macro_use]
extern crate bitflags;

use task::TASK_MANAGER;
use crate::sbi::shutdown;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

#[unsafe(no_mangle)]
fn rust_main(_hart_id: usize, device_tree_ptr: usize) {
    unsafe extern "C" {
        // safe fn stext(); // begin addr of text segment
        // safe fn etext(); // end addr of text segment
        // safe fn srodata(); // start addr of Read-Only data segment
        // safe fn erodata(); // end addr of Read-Only data ssegment
        // safe fn sdata(); // start addr of data segment
        // safe fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        // safe fn boot_stack_lower_bound(); // stack lower bound
        // safe fn boot_stack_top(); // stack top
    }
    
    clear_bss(sbss as usize, ebss as usize);
    logging::init_logger();
    debug!("[Kernel] Booting up on hart {}", _hart_id);
    debug!("[Kernel] Device Tree Pointer: {:#x}", device_tree_ptr);
    mm::heap_allocator::init_heap();
    logging::enable_heap_logging();
    io::init(device_tree_ptr as *const u8);
    mm::init();
    trap::init();
    // trap::enable_timer_interrupt();
    // timer::set_next_trigger();
    // loader::load_apps();
    TASK_MANAGER.run_next_app();
    // info!("[Kernel] No works to do, shutdown");
    // system_reset(Shutdown, NoReason);
    panic!("[Kernel][Assert] Unreachable");
}
#[unsafe(no_mangle)]
fn clear_bss(sbss: usize, ebss: usize){
    (sbss..ebss).for_each(|addr| {
        unsafe{(addr as *mut u8).write_volatile(0);}
    });
}
