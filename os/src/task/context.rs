use core::arch::{asm, global_asm};

use crate::trap::trap_return;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct TaskContext{
    pub ra: usize,
    pub sp: usize,
    // pub s: [usize;12]
}

impl TaskContext {
    pub fn new(user_kernel_sp: usize) -> Self {
        TaskContext { ra: trap_return as usize, sp: user_kernel_sp }
    }
    pub fn switch_to(&self) -> ! {
        unsafe{
            asm!("ld ra, {0}",
                "ld sp, {1}",
                "ret",
                in(reg) self.ra, in(reg) self.sp,
                options(noreturn))
        }
    }
}
// global_asm!(include_str!("switch.S"));


