//use crate::batch::{APP_BASE_ADDRESS, APP_SIZE_LIMIT,USER_STACK};

use crate::{mm::translated_byte_buffer, task::current_user_token};

const FD_STDOUT: usize = 1;
//const STACK_SIZE: usize = 0x1000;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            -1
        }
    }
}