#![no_std]
#![no_main]


#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32{
   println!("hello world"); 
   0
}

