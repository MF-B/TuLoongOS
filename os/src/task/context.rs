#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 10],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    UnInit,  // 未初始化
    Ready,   // 准备运行
    Running, // 正在运行
    Exited,  // 已退出
}
impl Default for TaskState {
    fn default() -> Self {
        TaskState::UnInit
    }
}

#[derive(Default, Clone, Copy)]
pub struct TaskControlBlock {
    pub context: TaskContext,
    pub state: TaskState,
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
