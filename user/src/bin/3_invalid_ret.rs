#![no_std]
#![no_main]

use core::arch::asm;


#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32{
    println!("Going to do a invalid ret from S mode, kernel should kill this application");
    unsafe{
        asm!("sret");
    }    
    println!("This shouldn't appear on the screen");
    0
}

