use riscv::register::sstatus::{self, Sstatus, SPP};


#[derive(Debug)]
#[repr(C)]
pub struct TrapContext{
    pub x: [usize;32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize
}
impl TrapContext{
    pub fn app_init_context(entry: usize, user_sp: usize, kernel_satp: usize, kernel_sp: usize, trap_handler: usize) -> Self{
        let mut csr_status = sstatus::read();
        csr_status.set_spp(SPP::User);
        unsafe{ sstatus::write(csr_status); }
        let mut cx = Self {
            x: [0; 32],
            sstatus: csr_status,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler: trap_handler
        };
        cx.set_sp(user_sp); // For first time running
        cx
    }

    fn set_sp(&mut self, sp: usize){
        self.x[2] = sp;
    }
}