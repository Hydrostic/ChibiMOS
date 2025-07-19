use core::arch::global_asm;

use crate::task::context::TaskContext;
global_asm!(include_str!("switch.S"));
unsafe extern "C"{
    pub fn __switch(next_task_cx_ptr: *const TaskContext) -> !;
}