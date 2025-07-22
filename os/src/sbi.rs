use core::arch::asm;

const SBI_EXT_SRST: usize = 0x53525354;
const SBI_EXT_DBCN: usize = 0x4442434E;
const SBI_EXT_TIME: usize = 0x54494D45;
const SBI_EXT_SRST_RESET: usize = 0x0;

const SBI_SRST_RESET_REASON_NONE: usize = 0x0;
const SBI_SRST_RESET_REASON_SYSFAIL: usize = 0x1;
const SBI_SRST_RESET_TYPE_SHUTDOWN: usize = 0x0;
const SBI_DBCN_CONSOLE_WRITE: usize = 0x0;
const SBI_TIME_SET_TIMER: usize = 0x0;
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
    sbi_call(SBI_EXT_TIME, SBI_TIME_SET_TIMER, timer, 0, 0);
}

/// use sbi call to putchar in console (qemu uart handler)

unsafe extern {
    fn ekernel();
}
pub fn putstr(content: &str){
    let ptr = content.as_ptr() as usize;
    // if ptr >= ekernel as usize && let Some(kms) = KERNEL_MEMORY_SET.exclusive_access().as_ref() {
    // }else{
        sbi_call(SBI_EXT_DBCN, SBI_DBCN_CONSOLE_WRITE,
                 content.len(),
                 ptr & u32::MAX as usize,
                 0);

    // }
}
pub fn putstr_debug(content: &str) {
    let ptr = content.as_ptr() as usize;
    sbi_call(SBI_EXT_DBCN, SBI_DBCN_CONSOLE_WRITE,
             content.len(),
             ptr & u32::MAX as usize,
             0);

}
pub fn shutdown(is_failure: bool) -> !{
    sbi_call(SBI_EXT_SRST, SBI_EXT_SRST_RESET, SBI_SRST_RESET_TYPE_SHUTDOWN, if is_failure { SBI_SRST_RESET_REASON_SYSFAIL } else { SBI_SRST_RESET_REASON_NONE }, 0);
    unreachable!();
}

