use core::{arch::asm, cell::SyncUnsafeCell};

// use crate::trap::context::TrapContext;


// const MAX_APP_NUM: usize = 64;
// const APP_MEM_LIMIT: usize = 64*1024;
// const APP_BASE_ADDRESS: usize = 0x80400000;

// const USER_STACK_SIZE: usize = 4096*2;
// const KERNEL_STACK_SIZE: usize = 4096*2;

// #[repr(align(4096))]
// struct KernelStack{
//     data: SyncUnsafeCell<[u8;KERNEL_STACK_SIZE * MAX_APP_NUM]>,
// }

// impl KernelStack{
//     fn get_sp(&self) -> usize{
//         self.data.get() as usize + KERNEL_STACK_SIZE * MAX_APP_NUM
//     }
//     fn get_app_sp(&self, app_id: usize) -> usize{
//         self.get_sp() - KERNEL_STACK_SIZE * app_id
//     }
//     fn push_context(&self, sp: usize, context: TrapContext) -> &'static mut TrapContext{
//         let ptr = (sp - core::mem::size_of::<TrapContext>()) as *mut TrapContext;

//         unsafe{ core::ptr::write(ptr, context) };

//         unsafe{ ptr.as_mut().unwrap() }
//     }
// }

// #[repr(align(4096))]
// struct UserStack{
//     // actually we don't operate user stack on kernel, so there is no need to wrap it with UnsafeCell
//     data: SyncUnsafeCell<[u8;USER_STACK_SIZE * MAX_APP_NUM]>
// }

// impl UserStack{
//     fn get_sp(&self) -> usize{
//         self.data.get() as usize + USER_STACK_SIZE * MAX_APP_NUM
//     }
//     fn get_app_sp(&self, app_id: usize) -> usize{
//         self.get_sp() - USER_STACK_SIZE * app_id
//     }
// }
// static KERNEL_STACK: KernelStack = KernelStack{ data: SyncUnsafeCell::new([0;KERNEL_STACK_SIZE * MAX_APP_NUM]) };
// static USER_STACK: UserStack = UserStack{ data: SyncUnsafeCell::new([0;USER_STACK_SIZE * MAX_APP_NUM]) };

// pub fn init_app_context(app_id: usize) -> usize{
//     core::ptr::from_mut(KERNEL_STACK.push_context(KERNEL_STACK.get_app_sp(app_id), TrapContext::app_init_context(
//         get_app_base(app_id),
//         USER_STACK.get_app_sp(app_id)
//     ))) as usize
// }

unsafe extern "C"{  fn _num_app(); }
pub fn get_app_num() -> usize{
    unsafe{ (_num_app as *const usize).read_volatile() }
}
pub fn get_app_data(app_id: usize) -> &'static [u8]{
    
    unsafe {
        let app_data_start_ptr = (_num_app as *const usize).add(1 + app_id).read_volatile();
        let app_data_end_ptr = (_num_app as *const usize).add(2 + app_id).read_volatile();
        let app_data = core::slice::from_raw_parts(app_data_start_ptr as *const u8, app_data_end_ptr - app_data_start_ptr);
        app_data
    }
}
// pub fn get_app_src(app_id: usize) -> (usize, usize){
//     (
//         unsafe { (_num_app as *const usize).add(1 + app_id).read_volatile() },
//         unsafe { (_num_app as *const usize).add(2 + app_id).read_volatile() },
//     )
// }
// pub fn get_app_base(app_id: usize) -> usize{
//     APP_BASE_ADDRESS + app_id * APP_MEM_LIMIT
// }
// pub fn load_apps(){
//     let app_num = get_app_num();
//     for i in 0..app_num{
//         let (app_src_start, app_src_end) = get_app_src(i);
//         let app_dst_start = APP_BASE_ADDRESS + i * APP_MEM_LIMIT; 
//         let app_dst = unsafe { core::slice::from_raw_parts_mut(app_dst_start as *mut u8, APP_MEM_LIMIT) };
//         app_dst.fill(0);
//         let app_src = unsafe{ core::slice::from_raw_parts(app_src_start as *const u8, app_src_end - app_src_start) }; 
//         app_dst[..app_src.len()].copy_from_slice(app_src);
//     }
//     unsafe {
//         asm!("fence.i");
//     }
// }