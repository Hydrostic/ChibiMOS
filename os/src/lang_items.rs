use core::panic::PanicInfo;

use alloc::format;

use crate::{println, sbi::shutdown};

#[panic_handler]
fn panic(info: &PanicInfo) -> !{
    let msg = info.message();
    if let Some(t) = info.location() {
        let s = format_args!("\x1b[41;37m[F]{}:{} System panicked: {}\x1b[0m\n", t.file(), t.line(), msg);
        crate::console::print(s);
    }else{
        println!("\x1b[41;37m[F]-:- System panicked: {}\x1b[0m\n", msg);
    }
    shutdown(true)
}