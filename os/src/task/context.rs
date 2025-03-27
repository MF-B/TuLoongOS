use crate::{loader::init_app_cx, mm::memory_set::MemorySet};
use crate::trap::trap_return;

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

#[derive(Clone)]
pub struct TaskControlBlock {
    pub context: TaskContext,
    pub status: TaskStatus,
    pub memory_set: MemorySet,    //新增的地址空间
    pub task_id: usize,
    //pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let status = TaskStatus::Ready; //准备指向状态
        let task_control_block = Self {
            context: TaskContext::goto_restore(init_app_cx(app_id, entry_point, user_sp)),
            status,
            //初始化任务上下文,参数为内核栈地址，内核栈存放的是trap上下文
            memory_set,
            task_id: app_id,
            //base_size: user_sp,
        };
        // prepare TrapContext in user space
        task_control_block
    }

    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
}

impl TaskContext {
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        unsafe extern "C" {
            fn __restore();
        }
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            s: [0; 10],
        }
    }
}
