use crate::trap::context::TrapContext;
use core::{arch::asm, cell::{RefCell, RefMut}};
use core::cell::UnsafeCell;

use lazy_static::lazy_static;
use log::info;
use sbi_rt::{system_reset, NoReason, Shutdown};

const MAX_APP_NUM: usize = 511;
const APP_MEM_LIMIT: usize = 64*1024;
const APP_BASE_ADDRESS: usize = 0x80400000;

const USER_STACK_SIZE: usize = 4096*2;
const KERNEL_STACK_SIZE: usize = 4096*2;

#[repr(align(4096))]
struct KernelStack{
    data: [u8;KERNEL_STACK_SIZE],
    // ptr: usize
}

impl KernelStack{
    fn get_sp(&self) -> usize{
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    // #[unsafe(no_mangle)]
    fn push_context(&self, context: TrapContext) -> &'static mut TrapContext{
        let ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

        unsafe{ core::ptr::write(ptr, context) };

        unsafe{ ptr.as_mut().unwrap() }
    }
}

#[repr(align(4096))]
struct UserStack{
    data: [u8;USER_STACK_SIZE]
}

impl UserStack{
    fn get_sp(&self) -> usize{
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}
static KERNEL_STACK: KernelStack = KernelStack{ data: [0;KERNEL_STACK_SIZE] };
static USER_STACK: UserStack = UserStack{ data: [0;USER_STACK_SIZE] };
pub struct AppManager{
    num: usize,
    current_id: usize,
    app_start: [usize;MAX_APP_NUM+1]   
}

pub struct SafeCell<T>{
    inner: RefCell<T>
}

unsafe impl<T> Sync for SafeCell<T>{}

impl<T> SafeCell<T>{
    pub fn new(v: T) -> Self{
        SafeCell{ inner: RefCell::new(v) }
    }

    pub fn exclusive_access(&self) -> RefMut<T>{
        self.inner.borrow_mut()
    }
}

lazy_static!{
    pub static ref APP_MANAGER: SafeCell<AppManager> = {
        SafeCell::new({
            unsafe extern "C"{
                fn _num_app();
            }
            let num_ptr = _num_app as *const usize;
            let num = unsafe{ num_ptr.read_volatile() };
            info!("[AppManager] {} applications in total", num);
            let mut manager = AppManager{
                num,
                current_id: 0,
                app_start: [0; MAX_APP_NUM + 1],
            };

            manager.app_start[..num+1].copy_from_slice(unsafe{ core::slice::from_raw_parts(num_ptr.add(1), num + 1) });

            manager
        })
    };
}
impl AppManager{
    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num{
            panic!("[AppManager] Inexist app_id");
        }
        info!("[AppManager] Going to load app {}", app_id);
        unsafe{
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut usize, APP_MEM_LIMIT)
                .fill(0);
            let app_program = core::slice::from_raw_parts(self.app_start[app_id] as *mut usize, 
                self.app_start[app_id+1] - self.app_start[app_id]);
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut usize, app_program.len())
                .copy_from_slice(app_program);
            asm!("fence.i");
        }
    }

    pub fn get_current_app_id(&self) -> usize{
        self.current_id
    }

    pub fn move_to_next_app(&mut self) -> usize{
        if self.current_id == self.num - 1{
            info!("[AppManager] All applications finished running, shutdown");
            system_reset(Shutdown, NoReason);
        }
        self.current_id = self.current_id + 1;
        self.current_id
    }
}

fn run_app() -> !{
    unsafe extern "C"{ fn __restore(cx_addr: usize); }
    unsafe{
        // TODO
        __restore(KERNEL_STACK.push_context(
            TrapContext::app_init_context(APP_BASE_ADDRESS, USER_STACK.get_sp()) 
        ) as *const _ as usize);
    }
    panic!("[Kernel] Shouldn't reach here");
}

pub fn start_run_apps() -> !{
    {
        let app_manager = APP_MANAGER.exclusive_access();
        unsafe{
            app_manager.load_app(0);
        }
    }
    run_app()
}pub fn run_next_app() -> !{
    {
        let mut app_manager = APP_MANAGER.exclusive_access();
        let app_id = app_manager.move_to_next_app();
        unsafe{
            app_manager.load_app(app_id);
        }
    }
    run_app()
}