#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 10],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    UnInit,  // 未初始化
    Ready,   // 准备运行
    Running, // 正在运行
    Exited,  // 已退出
}
impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::UnInit
    }
}

#[derive(Default, Clone, Copy)]
pub struct TaskControlBlock {
    pub context: TaskContext,
    pub status: TaskStatus,
}

impl TaskContext {
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        unsafe extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 10],
        }
    }
}
