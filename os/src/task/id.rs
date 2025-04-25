use alloc::{sync::Weak, sync::Arc, vec::Vec};
use lazy_static::*;

use crate::{config::{PAGE_SIZE, USER_STACK_SIZE}, mm::{frame_alloc, memory_set::MapPermission, FrameTracker, PhysAddr, VirtAddr}, sync::UPSafeCell, trap::TrapFrame};

use super::process::ProcessControlBlock;


#[derive(Clone, Debug)]
pub struct KernelStack {
    frame:FrameTracker,
}

pub fn kstack_alloc() -> KernelStack {
    frame_alloc().map(|frame| KernelStack{frame}).unwrap()
}

impl KernelStack {
    pub fn push_context(&self, trap_cx: TrapFrame) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
        unsafe {
            *trap_cx_ptr = trap_cx;
        };
        trap_cx_ptr as usize
    }
    fn get_sp(&self) -> usize {
        let sp: usize = PhysAddr::from(self.frame.ppn).into();
        sp + PAGE_SIZE
    }

    pub fn get_mut<T>(&self) -> &'static mut T {
        let sp = self.get_sp();
        let sp = sp - core::mem::size_of::<T>();
        unsafe {
            (sp as *mut T).as_mut().unwrap()
        }
    }

    #[allow(unused)]
    pub fn get_trap_cx(&self) -> &'static mut TrapFrame {
        self.get_mut::<TrapFrame>()
    }

    pub fn get_trap_addr(&self) -> usize {
        self.get_sp() - core::mem::size_of::<TrapFrame>()
    }

    pub fn init_app_cx(&self, entry_point: usize, user_sp: usize) -> usize {
        self.push_context(TrapFrame::app_init_context(entry_point, user_sp))
    }

    pub fn copy_from_existed(&self, old_stack: &KernelStack) {
        let old_cx = old_stack.get_mut::<TrapFrame>();
        let new_cx = self.get_mut::<TrapFrame>();
        new_cx.crmd = old_cx.crmd;
        new_cx.era = old_cx.era;
        new_cx.regs = old_cx.regs;
    }
}


pub struct RecycleAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl RecycleAllocator {
    pub fn new() -> Self {
        RecycleAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            self.current += 1;
            self.current - 1
        }
    }
    pub fn dealloc(&mut self, id: usize) {
        assert!(id < self.current);
        assert!(
            !self.recycled.iter().any(|i| *i == id),
            "id {} has been deallocated!",
            id
        );
        self.recycled.push(id);
    }
}

pub struct PidHandle(pub usize);

lazy_static! {
    static ref PID_ALLOCATOR: UPSafeCell<RecycleAllocator> =
        unsafe { UPSafeCell::new(RecycleAllocator::new()) };
}

pub fn pid_alloc() -> PidHandle {
    PidHandle(PID_ALLOCATOR.exclusive_access().alloc())
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

pub struct TaskUserRes {
    pub tid: usize,
    pub ustack_base: usize,
    pub process: Weak<ProcessControlBlock>,
}

fn ustack_bottom_from_tid(ustack_base: usize, tid: usize) -> usize {
    ustack_base + tid * (PAGE_SIZE + USER_STACK_SIZE)
}

impl TaskUserRes {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let tid = process.inner_exclusive_access().alloc_tid();
        let task_user_res = Self {
            tid,
            ustack_base,
            process: Arc::downgrade(&process),
        };
        if alloc_user_res {
            task_user_res.alloc_user_res();
        }
        task_user_res
    }

    /// 在进程地址空间中实际映射线程的用户栈和 Trap 上下文。
    pub fn alloc_user_res(&self) {
        let process = self.process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        // alloc user stack
        let ustack_bottom = ustack_bottom_from_tid(self.ustack_base, self.tid);
        let ustack_top = ustack_bottom + USER_STACK_SIZE;
        process_inner.memory_set.insert_framed_area(
            ustack_bottom.into(),
            ustack_top.into(),
            MapPermission::default() | MapPermission::W | MapPermission::PLV0 | MapPermission::PLV1,
        );
    }

    fn dealloc_user_res(&self) {
        // dealloc tid
        let process = self.process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        // dealloc ustack manually
        let ustack_bottom_va: VirtAddr = ustack_bottom_from_tid(self.ustack_base, self.tid).into();
        process_inner
            .memory_set
            .remove_area_with_start_vpn(ustack_bottom_va.into());
    }
    pub fn dealloc_tid(&self) {
        let process = self.process.upgrade().unwrap();
        let mut process_inner = process.inner_exclusive_access();
        process_inner.dealloc_tid(self.tid);
    }

    pub fn ustack_top(&self) -> usize {
        ustack_bottom_from_tid(self.ustack_base, self.tid) + USER_STACK_SIZE
    }

    pub fn ustack_base(&self) -> usize {
        self.ustack_base
    }
}

impl Drop for TaskUserRes {
    fn drop(&mut self) {
        self.dealloc_tid();
        self.dealloc_user_res();
    }
}