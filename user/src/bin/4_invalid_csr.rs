#![no_std]
#![no_main]

use user_lib::{get_time_us, yield_now};

#[macro_use]
extern crate user_lib;

use riscv::register::sstatus::{self, SPP};

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Try to access privileged CSR in U Mode");
    println!("Kernel should kill this application!");
    unsafe {
        sstatus::set_spp(SPP::User);
    }
    0
    // let current_timer = get_time_us();
    // let wait_for = current_timer + 3000;
    // while get_time_us() < wait_for {
    //     yield_now();
    // }
    // println!("Test sleep OK!");
    // 0
}