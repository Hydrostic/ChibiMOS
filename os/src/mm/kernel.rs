use core::cell::OnceCell;
use lazy_static::lazy_static;
use log::debug;
use crate::helper::cell::SingleThreadSafeCell;
use crate::mm::address;
use crate::mm::memory_structure::{AddressIterator, MemoryArea, MemoryAreaPermissions, MemoryAreaType, MemorySet, MemoryStructureError};

#[allow(dead_code)]
unsafe extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss_with_stack();
    fn ebss();
    fn ekernel();
    fn strampoline();
}

pub struct KernelMemoryManager {
    memory_set: OnceCell<MemorySet>,
    virtual_enabled: bool
}

lazy_static!{
    pub static ref KERNEL_MEMORY_MANAGER: SingleThreadSafeCell<KernelMemoryManager> = SingleThreadSafeCell::new(KernelMemoryManager {
        memory_set: OnceCell::new(),
        virtual_enabled: false
    });
}
impl KernelMemoryManager {
    pub fn virtual_enabled(&self) -> bool {
        self.virtual_enabled
    }
    pub fn is_identical_address(&self, address: usize) -> bool {
        address < ekernel as usize
    }
    pub fn map_stack_for_process_syscall(&mut self, id: usize) -> Result<(), MemoryStructureError> {
        let stack_range = address::kernel_stack_position(id);

        self.memory_set.get_mut().unwrap().push(
            MemoryArea::new(stack_range.start,
                            stack_range.end,
                            MemoryAreaType::Framed,
                            MemoryAreaPermissions::R | MemoryAreaPermissions::W
            )?, None
        )?;
        
        Ok(())
    }

    pub fn translate_byte_buffer(&self, ptr: *const u8, len: usize) -> Result<AddressIterator, MemoryStructureError> {
        self.memory_set.get().unwrap().translate_byte_buffer((ptr as usize).into(), len)
    }
    pub fn init(&mut self, start: usize, end: usize) {
        let mut set = MemorySet::new().expect("[MM] Failed to create kernel memory set");

        set.map_trampoline().expect("Failed to map trampoline");
        debug!("[MM] Kernel segment .text [{:#x}, {:#x})", stext as usize, etext as usize);
        debug!("[MM] Kernel segment .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        debug!("[MM] Kernel segment .data [{:#x}, {:#x})", sdata as usize, edata as usize);
        debug!("[MM] Kernel segment .bss(include stack) [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);

        set.push(MemoryArea::new((stext as usize).into(), (etext as usize).into(),
                                 MemoryAreaType::Identical,
                                 MemoryAreaPermissions::R | MemoryAreaPermissions::X).unwrap(), None).expect("[MM] Failed to push .text area");

        set.push(MemoryArea::new((srodata as usize).into(), (erodata as usize).into(),
                                 MemoryAreaType::Identical,
                                 MemoryAreaPermissions::R).unwrap(), None).expect("[MM] Failed to push .rodata area");

        set.push(MemoryArea::new((sdata as usize).into(), (edata as usize).into(),
                                 MemoryAreaType::Identical,
                                 MemoryAreaPermissions::R | MemoryAreaPermissions::W).unwrap(), None).expect("[MM] Failed to push .data area");

        set.push(MemoryArea::new((sbss_with_stack as usize).into(), (ebss as usize).into(),
                                 MemoryAreaType::Identical,
                                 MemoryAreaPermissions::R | MemoryAreaPermissions::W).unwrap(), None).expect("[MM] Failed to push .bss area");

        set.push(MemoryArea::new((ekernel as usize).into(), end.into(),
                                 MemoryAreaType::Identical,
                                 MemoryAreaPermissions::R | MemoryAreaPermissions::W).unwrap(), None).expect("[MM] Failed to push left area");
        set.activate();
        self.memory_set.set(set).map_err(|_| ()).expect("[MM] Kernel memory set already initialized");
    }
    
    pub fn token(&self) -> usize {
        self.memory_set.get().unwrap().token()
    }

}
