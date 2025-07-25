#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{get_time_us, yield_now};

#[unsafe(no_mangle)]
fn main() -> i32 {
    let current_timer = get_time_us();
    let wait_for = current_timer + 3000;
    while get_time_us() < wait_for {
        yield_now();
    }
    println!("Test sleep OK!");
    0
}