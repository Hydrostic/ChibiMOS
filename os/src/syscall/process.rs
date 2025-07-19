use log::info;

// use crate::batch::{APP_MANAGER, self};
use crate::{task::TASK_MANAGER, timer::get_time_us};
pub fn sys_exit(xstate: i32) -> !{
    let app_id = {
        TASK_MANAGER.get_current_app_id()
    };
    info!("[Kernel] Application {} exited with code {}", app_id, xstate);
    TASK_MANAGER.set_exit();
    TASK_MANAGER.run_next_app();
}

pub fn sys_yield() -> !{
    TASK_MANAGER.suspend();
    TASK_MANAGER.run_next_app();    
}

pub fn sys_get_time() -> isize{
    get_time_us() as isize
}