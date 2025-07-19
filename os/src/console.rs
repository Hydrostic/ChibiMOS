use core::fmt::{self, Write};
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
        sbi::putstr(s);
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