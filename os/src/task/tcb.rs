use crate::mm::address::{self, PhysPageNumber, TRAP_CONTEXT};
use crate::mm::memory_structure::{self, MemoryArea, MemoryAreaPermissions, MemoryAreaType, MemorySet, MemoryStructureError};
use crate::mm::kernel::KERNEL_MEMORY_MANAGER;
use crate::trap::context::TrapContext;
use crate::trap::{trap_handler, trap_return};
use super::context::TaskContext;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Failed to initialize memory for task: {0}")]
    FailedToInitializeMemory(#[from] MemoryStructureError)
}
pub struct TaskControlBlock{
    task_status: TaskStatus,
    pub task_cx: TaskContext,
    memory_set: MemorySet,
    task_cx_ppn: PhysPageNumber,
    base_size: usize
}
impl TaskControlBlock{
    pub fn new(app_id: usize, elf_data: &[u8]) -> Result<Self, TaskError> {
        let (memory_set, user_sp, entry) = memory_structure::new_elf_memory_set(app_id, elf_data)?;
        let task_cx_ppn = memory_set.translate(TRAP_CONTEXT.into())?;

        let kernel_stack_range = address::kernel_stack_position(app_id);
        KERNEL_MEMORY_MANAGER.exclusive_access().map_stack_for_process_syscall(app_id)?;
        *task_cx_ppn.get_mut::<TrapContext>() = TrapContext::app_init_context(
            entry, 
            user_sp, 
            KERNEL_MEMORY_MANAGER.exclusive_access().token(),
            kernel_stack_range.end.into(),
            trap_handler as usize
        );
        Ok(Self {
            task_status: TaskStatus::Ready,
            task_cx: TaskContext::new(kernel_stack_range.end.into()),
            memory_set,
            base_size: user_sp,
            task_cx_ppn
        })
    }
    pub fn suspend(&mut self) {
        self.task_status = TaskStatus::Ready;
    }
    pub fn set_run(&mut self) {
        self.task_status = TaskStatus::Running;
        // self.task_cx.switch_to();
    }
    pub fn set_exit(&mut self) {
        self.task_status = TaskStatus::Exited;
    }

    pub fn status(&self) -> TaskStatus {
        self.task_status
    }


    pub fn satp_token(&self) -> usize {
        self.memory_set.token()
    }

    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        self.task_cx_ppn.get_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus{
    Ready,
    Running,
    Exited
}