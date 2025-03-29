use core::arch::asm;
use core::cell::RefMut;

use alloc::sync::Weak;
use alloc::sync::Arc;
use alloc::vec::Vec;
use log::info;
use loongArch64::register::pgdl;

use crate::config::PAGE_SIZE_BITS;
use crate::sync::UPSafeCell;
use crate::mm::memory_set::MemorySet;

use super::{pid::{alloc_pid, KernelStack, PidHandle}, task::{TaskContext, TaskStatus}};



pub struct ProcessControlBlock {
    // 不可变部分immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    pub base_size: usize,
    pub task_context: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,

    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,

    pub exit_code: i32,
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8]) -> Self {
        let pid = alloc_pid();
        let kernel_stack = KernelStack::new();

        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let task_context = TaskContext::goto_trap_return(kernel_stack.init_app_cx(entry_point, user_sp));
        let task_status = TaskStatus::Ready;

        Self { 
            pid, 
            kernel_stack, 
            inner: unsafe { UPSafeCell::new(ProcessControlBlockInner {
                base_size: user_sp,
                task_context,
                task_status,
                memory_set,
                parent: None,
                children: Vec::new(),
                exit_code: 0,
            }) }, 
        }
    }

    pub fn get_trap_cx(&self) -> usize {
        self.kernel_stack.get_trap_cx()
    }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn exec(&self, elf_data: &[u8]) {
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let mut inner = self.inner_exclusive_access();
        inner.memory_set = memory_set;
        self.kernel_stack.init_app_cx(entry_point, user_sp);
        let pid = self.getpid();
        unsafe {
            asm!("invtlb 0x4,{},$r0",in(reg) pid);
        }
        let pgd = inner.get_user_token() << PAGE_SIZE_BITS;
        pgdl::set_base(pgd);
    }
    pub fn fork(self: &Arc<ProcessControlBlock>) -> Arc<ProcessControlBlock> {
        let pid = alloc_pid();

        info!("fork process: pid: {}, parent pid: {}", pid.0, self.getpid());
        let mut parent_pcb = self.inner_exclusive_access();

        // 新pcb内容设置
        let c_kernel_stack = KernelStack::new();
        c_kernel_stack.copy_from_existed(&self.kernel_stack);
        // let old_trap_cx = self.kernel_stack.get_mut::<TrapFrame>();
        // let new_trap_cx = c_kernel_stack.get_mut::<TrapFrame>();
        // info!("old_trap_cx: {:#?}, new_trap_cx: {:#?}", old_trap_cx, new_trap_cx);

        let c_base_size = parent_pcb.base_size;
        let c_task_context = TaskContext::goto_trap_return(c_kernel_stack.get_trap_cx());
        let c_memory_set = MemorySet::from_existed_process(&parent_pcb.memory_set);
        let child_pcb = Arc::new(ProcessControlBlock {
            pid,
            kernel_stack: c_kernel_stack,
            inner: unsafe { UPSafeCell::new(ProcessControlBlockInner {
                base_size: c_base_size,
                task_context: c_task_context,
                task_status: TaskStatus::Ready,
                memory_set: c_memory_set,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
            }) },
        });

        // 父pcb内容设置
        parent_pcb.children.push(child_pcb.clone());

        child_pcb
    }
}

impl ProcessControlBlockInner {
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn is_zombie(&self) -> bool {
        self.get_status() == TaskStatus::Zombie
    }
}


