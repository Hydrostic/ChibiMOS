#![no_std]
#![no_main]

use core::ptr;


#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32{
    println!("Going to visit invalid address, kernel should kill this application");
    unsafe{
        ptr::null_mut::<u8>().write_volatile(0);
    }    
    println!("This shouldn't appear on the screen");
    0
}

