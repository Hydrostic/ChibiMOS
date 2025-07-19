mod fs;
mod process;


pub fn syscall(id: usize, args: [usize;3]) -> isize{
    if let Some(syscall_type) = SyscallType::from_number(id){
        match syscall_type{
            SyscallType::SysWrite => fs::sys_write(args[0], args[1] as *const u8, args[2]),
            SyscallType::SysExit => process::sys_exit(args[0] as i32),
            SyscallType::SysYield => process::sys_yield(),
            SyscallType::SysGetTime => process::sys_get_time()
        }
    }else{
        -1
    }
}

#[repr(usize)]
pub enum SyscallType{
    SysWrite = 64,
    SysExit = 93,
    SysYield = 124,
    SysGetTime = 169
}

impl SyscallType{
    pub fn from_number(id: usize) -> Option<Self>{
        match id{
            64 => Some(Self::SysWrite),
            93 => Some(Self::SysExit),
            124 => Some(Self::SysYield),
            169 => Some(Self::SysGetTime),
            _ => None
        }
    }
}