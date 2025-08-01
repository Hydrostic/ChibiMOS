use core::fmt::{self, Write};

use crate::write;


const STDOUT: usize = 1;

pub struct Stdout;

impl Write for Stdout{
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if write(STDOUT, s.as_bytes()) == -1{
            Err(fmt::Error)
        }else{
            Ok(())
        }
    }
}
pub fn print(args: core::fmt::Arguments){
    Stdout.write_fmt(args).unwrap();
}

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
