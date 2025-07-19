use alloc::format;
use alloc::string::ToString;
use core::arch::asm;
use crate::mm::KERNEL_MEMORY_SET;

const SBI_EXT_SRST: usize = 0x53525354;
const SBI_EXT_DBCN: usize = 0x4442434E;
const SBI_LEGACY_SET_TIMER: usize = 0;
const SBI_SRST_RESET_REASON_NONE: usize = 0x0;
const SBI_SRST_RESET_REASON_SYSFAIL: usize = 0x1;
const SBI_SRST_RESET_TYPE_SHUTDOWN: usize = 0x0;

const SBI_DBCN_CONSOLE_WRITE: usize = 0x0;

#[inline(always)]
/// general sbi call
fn sbi_call(ext_id: usize, func_id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut ret;
    unsafe {
        asm!(
        "ecall",
        inlateout("x10") arg0 => ret,
        in("x11") arg1,
        in("x12") arg2,
        in("x16") func_id,
        in("x17") ext_id,
        );
    }
    ret
}

/// use sbi call to set timer
pub fn set_timer(timer: usize) {
    sbi_call(SBI_LEGACY_SET_TIMER, 0, timer, 0, 0);
}

/// use sbi call to putchar in console (qemu uart handler)

pub fn sleep(){
    // sbi_rt::set_timer();

    // unsafe{asm!("wfi");}
}
unsafe extern {
    fn ekernel();
}
pub fn putstr(content: &str){
    let ptr = content.as_ptr() as usize;
    if ptr >= ekernel as usize && let Some(kms) = KERNEL_MEMORY_SET.exclusive_access().as_ref() {
        if let Ok(addr_arr) = kms
            .translate_byte_buffer(ptr.into(), content.len()) {
            addr_arr.for_each(|(addr, len)| {
                sbi_call(SBI_EXT_DBCN, SBI_DBCN_CONSOLE_WRITE,
                         len,
                         addr.into(),
                         0);
            });
        }
    }else{
        sbi_call(SBI_EXT_DBCN, SBI_DBCN_CONSOLE_WRITE,
                 content.len(),
                 ptr & u32::MAX as usize,
                 0);

    }
}
pub fn putstr_debug(content: &str) {
    let ptr = content.as_ptr() as usize;
    sbi_call(SBI_EXT_DBCN, SBI_DBCN_CONSOLE_WRITE,
             content.len(),
             ptr & u32::MAX as usize,
             0);

}
pub fn shutdown(is_failure: bool) -> !{
    sbi_call(SBI_EXT_SRST, if is_failure { SBI_SRST_RESET_REASON_SYSFAIL } else { SBI_SRST_RESET_REASON_NONE }, 0, 0, 0);
    unreachable!();
}


// pub fn set_timer(timer: u64){
//     sbi_rt::set_timer(timer as u64);
// }