use core::arch::global_asm;

use super::task::TaskContext;

global_asm!(include_str!("switch.S"));

unsafe extern "C" {
    pub fn __switch(
        current_task_cx_ptr: *mut TaskContext, 
        next: *const TaskContext,
    );
}