use core::fmt::{self, Write};
use core::slice;
use log::error;
use crate::mm::kernel::KERNEL_MEMORY_MANAGER;
use crate::sbi;
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}


pub struct Stdout;

impl fmt::Write for Stdout{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let km = KERNEL_MEMORY_MANAGER.exclusive_access();
        if km.is_identical_address(s.as_ptr() as usize) {
            sbi::putstr(s);
        } else {
            match km.translate_byte_buffer(s.as_ptr(), s.len()){
                Err(e) => {
                    sbi::putstr("\u{25A1}")
                },
                Ok(addr_iter) => addr_iter.for_each(|(addr, len)| sbi::putstr(unsafe{
                    str::from_utf8_unchecked(slice::from_raw_parts(Into::<usize>::into(addr) as *const u8, len))
                }))
            };
        }
        Ok(())
    }
}


pub fn print(args: core::fmt::Arguments){
    let _ = Stdout.write_fmt(args);
}

macro_rules! with_color {
    ($args: ident, $color_code: ident) => {
        format_args!("\x1b[{}m{}\x1b[0m", $color_code, $args)
    };
}
pub fn print_with_color(args: core::fmt::Arguments, color: u8){
    let _ = Stdout.write_fmt(with_color!(args, color));
}