//use crate::batch::{APP_BASE_ADDRESS, APP_SIZE_LIMIT,USER_STACK};

const FD_STDOUT: usize = 1;
//const STACK_SIZE: usize = 0x1000;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            -1
        }
    }
}