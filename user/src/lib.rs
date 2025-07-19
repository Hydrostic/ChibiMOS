#![no_std]
#![feature(linkage)]

mod syscall;
#[macro_use]
pub mod console;
mod lang_items;

#[unsafe(link_section = ".text.entry")]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("test");
    // clear_bss();
    exit(main());
    panic!("Unreachable after sys_exit");
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32{
    panic!("Entry `main` not found");
}

// #[unsafe(no_mangle)]
// fn clear_bss(){
    // unsafe extern "C"{
        // fn start_bss();
        // fn end_bss();
    // }
    // (start_bss as usize..end_bss as usize).for_each(|addr| {
        // unsafe{(addr as *mut u8).write_volatile(0);}
    // });
// }

use syscall::*;

pub fn write(fd: usize, buf: &[u8]) -> isize { sys_write(fd, buf) }
pub fn exit(exit_code: i32) -> isize { sys_exit(exit_code) }
pub fn yield_now() -> isize{ sys_yield() }
pub fn get_time_us() -> isize{ sys_get_time() }