use alloc::vec::Vec;

use crate::{mm::page_table::PageTable, task::TASK_MANAGER};

const STDOUT_FD: usize = 1;


pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize{
    match fd{
        STDOUT_FD => {
            let buffers = match PageTable::translate_byte_buffer(TASK_MANAGER.get_current_satp_token(), buf, len){
                Ok(buffers) => buffers,
                Err(_) => return -1,
            };
            let buffers = buffers.iter().map(|slice| core::str::from_utf8(slice))
                .collect::<Result<Vec<&str>, _>>();
            match buffers {
                Ok(s) => {
                    let len = s.iter().map(|s| s.len()).sum::<usize>();
                    for str in s {
                        print!("{}", str);
                    }
                    len as isize
                },
                Err(_) => -1,
            }
        },
        _ => {
            -1
        }
        
    }
}