use core::arch::global_asm;

use super::context::TaskContext;

global_asm!(include_str!("task.S"));

unsafe extern "C" {
    pub fn __switch(current_task_cx_ptr: *mut TaskContext, next: *const TaskContext
    );
}