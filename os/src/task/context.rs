use crate::trap::trap_return;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 10],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaskStatus {
    UnInit,  // 未初始化
    Ready,   // 准备运行
    Running, // 正在运行
    Blocking,
}
impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::UnInit
    }
}

impl TaskContext {
    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 10],
        }
    }
}



