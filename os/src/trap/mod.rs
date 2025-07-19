pub mod context;

use core::arch::{asm, global_asm};
use context::TrapContext;
// use crate::{batch::{self, APP_MANAGER}, syscall::syscall};
use crate::{mm::address::{TRAMPOLINE, TRAP_CONTEXT}, syscall::syscall, task::{tcb::TaskControlBlock, TASK_MANAGER}, timer};
use riscv::{interrupt::{supervisor::Interrupt, Exception}, register::{satp, scause, sie, stval, stvec::{self, Stvec, TrapMode}}};


global_asm!(include_str!("trap.S"));


fn set_user_trap(){
    unsafe extern "C"{ fn __alltraps(); }
    let mut s = Stvec::from_bits(0);
    s.set_address(TRAMPOLINE as usize);
    s.set_trap_mode(TrapMode::Direct);
    unsafe{
        stvec::write(s);
    }
}
fn set_kernel_trap(){
    let mut s = Stvec::from_bits(0);
    s.set_address(trap_from_kernel as usize);
    s.set_trap_mode(TrapMode::Direct);
    unsafe{
        stvec::write(s);
    }
}
pub fn init() {
    set_kernel_trap();
}
// TODO: 检查计时器终端是否有冲突
// pub fn get_restore_fn() -> usize{
//     unsafe extern "C"{ fn __restore(); }
//     __restore as usize
// }
// 错误4：参数
#[unsafe(no_mangle)]
pub fn trap_handler() -> !{
    unsafe { asm!(".align 4") };
    set_kernel_trap();
    let scause = scause::read();
    let _stval = stval::read();
    let cx = TASK_MANAGER.get_current_trap_context();
    if scause.is_interrupt(){
        panic!("[Kernel] Currently Interrupts are not supported");
    }
    // Now the scause should be exceptions
    // 注：感觉这种 `try_into` 的方式还挺不错的，下次可以学习下
    match scause.cause().try_into::<riscv::interrupt::supervisor::Interrupt, _>().unwrap(){
        scause::Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        },
        scause::Trap::Exception(e) => if let Ok(msg) = e.try_get(){
            let app_id = {
                TASK_MANAGER.get_current_app_id()
            };
            log::error!("[Kernel] {} in application {} at {:#x}, killed", msg, app_id, cx.sepc);
            TASK_MANAGER.set_exit();
            TASK_MANAGER.run_next_app();
        },
        scause::Trap::Interrupt(Interrupt::SupervisorTimer) => {
            timer::set_next_trigger();
            TASK_MANAGER.suspend();
            TASK_MANAGER.run_next_app();
        }
        other => panic!("[Kernel] Current category of exception hasn't implemented: {:?}", other)
    }

    trap_return();
}
#[unsafe(no_mangle)]
// #[align(4)] 错误2：未加 align
pub fn trap_return() -> !{
    unsafe { asm!(".align 4") };
    set_user_trap();
    unsafe extern "C" {
        fn __restore();
        fn __alltraps();
    }
    let satp_token = TASK_MANAGER.get_current_satp_token();
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe{
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") TRAP_CONTEXT,
            in("a1") satp_token,
            options(noreturn))
    }
}
trait MsgHelper{
    fn try_get(&self) -> Result<&'static str, ()>;
}

impl MsgHelper for Exception{
    fn try_get(&self) -> Result<&'static str, ()> {
        match self{
            Exception::InstructionMisaligned => Ok("InstructionMisaligned"),
            Exception::InstructionFault => Ok("InstructionFault"),
            Exception::IllegalInstruction => Ok("IllegalInstruction"),
            Exception::Breakpoint => Err(()), // Shouldn't process it as an error and output msg
            Exception::LoadMisaligned => Ok("LoadMisaligned"),
            Exception::LoadFault => Ok("LoadFault"),
            Exception::StoreMisaligned => Ok("StoreMisaligned"),
            Exception::StoreFault => Ok("StoreFault"),
            Exception::UserEnvCall => Err(()),
            Exception::SupervisorEnvCall => Err(()),
            Exception::MachineEnvCall => Err(()),
            Exception::InstructionPageFault => Ok("InstructionPageFault"),
            Exception::LoadPageFault => Ok("LoadPageFault"),
            Exception::StorePageFault => Ok("StorePageFault"),
        }
    }
}

pub fn enable_timer_interrupt(){
    unsafe{
        sie::set_stimer();
    }
}
#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    unsafe{ asm!(".align 4"); }
    panic!("a trap from kernel!");
}