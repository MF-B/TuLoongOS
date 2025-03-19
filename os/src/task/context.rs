#[repr(C)]
#[derive(Default,Copy,Clone)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 10],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    Ready,
    Blocked,
}
impl Default for TaskState {
    fn default() -> Self {
        TaskState::Ready
    }
}


#[derive(Default,Clone, Copy)]
pub struct TaskControlBlock {
    pub context: TaskContext,
    pub state: TaskState,
}
