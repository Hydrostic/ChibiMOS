use core::cell::SyncUnsafeCell;
use alloc::{collections::btree_map::BTreeMap, vec::Vec};
use lazy_static::lazy_static;
use crate::{helper::cell::SingleThreadSafeCell, loader, sbi::shutdown, task::{switch::__switch, tcb::{TaskControlBlock, TaskStatus}}, trap::{context::TrapContext, trap_return}};
mod context;
mod switch;
pub(crate) mod tcb;

const MAX_TASK_NUM: usize = 64;
pub struct TaskManager{
    inner: SingleThreadSafeCell<_TaskManager>
}

lazy_static!{
    pub static ref TASK_MANAGER: TaskManager = TaskManager::new(loader::get_app_num());
}
impl TaskManager{
    pub fn new(app_num: usize) -> Self{
        let manager = {
            let mut control_blocks = BTreeMap::new();
            (0..app_num).for_each(|i| {
                let app_data = loader::get_app_data(i);
                match TaskControlBlock::new(i, app_data) {
                    Ok(tcb) => {
                        control_blocks.insert(i, tcb);
                    },
                    Err(e) => {
                        log::error!("[TaskManager] Failed to create TaskControlBlock for app {}: {}, skipping", i, e);
                    }
                }
            });
            _TaskManager { num: app_num, current_id: 0, control_blocks }
        };
        TaskManager{
            inner: SingleThreadSafeCell::new(manager)
        }
    }


    pub fn run_next_app(&self) -> !{
        if let Some(t) = self.find_next_app(){
            let mut manager;
            manager = self.inner.exclusive_access();
            // let prev_id = manager.current_id;
            manager.current_id = t;
            let tcb = &mut manager.control_blocks.get_mut(&t).unwrap();
            tcb.set_run();
            let satp_token = tcb.satp_token();
            let ptr = &mut tcb.task_cx as *mut _;
            drop(manager);
            unsafe{ __switch(ptr); }
        }else{
            log::info!("[TaskManager] All applications finished running, shutdown");
            shutdown(false);
        }
    }
    fn find_next_app(&self) -> Option<usize>{
        let manager;
        manager = self.inner.exclusive_access();
        for (i, tcb) in manager.control_blocks.iter(){
            if tcb.status() == TaskStatus::Ready {
                return Some(*i);
            }
        }
        None
    }

    pub fn get_current_app_id(&self) -> usize{
        let manager;
        manager = self.inner.exclusive_access();
        manager.current_id
    }
    pub fn get_current_trap_context(&self) -> &'static mut TrapContext {
        let manager;
        manager = self.inner.exclusive_access();
        let current_id = manager.current_id;
        manager.control_blocks.get(&current_id).unwrap().get_trap_context()
    }
    pub fn get_current_satp_token(&self) -> usize {
        let manager;
        manager = self.inner.exclusive_access();
        let current_id = manager.current_id;
        manager.control_blocks.get(&current_id).unwrap().satp_token()
    }
    pub fn get_current_kernel_sp(&self) -> usize {
        let manager;
        manager = self.inner.exclusive_access();
        let current_id = manager.current_id;
        manager.control_blocks.get(&current_id).unwrap().task_cx.sp
    }
    pub fn suspend(&self) {
        let mut manager;
        manager = self.inner.exclusive_access();
        let current_id = manager.current_id;
        manager.control_blocks.get_mut(&current_id).unwrap().suspend();
    }
    pub fn set_exit(&self) {
        let mut manager;
        manager = self.inner.exclusive_access();
        let current_id = manager.current_id;
        manager.control_blocks.get_mut(&current_id).unwrap().set_exit();
    }

    pub fn suspend_and_run_next(&self) -> ! {
        let mut manager;
        manager = self.inner.exclusive_access();
        let current_id = manager.current_id;
        if let Some(tcb) = manager.control_blocks.get_mut(&current_id) {
            tcb.suspend();
        }
        self.run_next_app();
    }
}

struct _TaskManager{
    num: usize,
    current_id: usize,
    control_blocks: BTreeMap<usize, TaskControlBlock>
}



