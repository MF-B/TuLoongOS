use alloc::vec::Vec;
use lazy_static::*;

use crate::{config::PAGE_SIZE, mm::{frame_alloc, FrameTracker, PhysAddr}, sync::UPSafeCell, trap::TrapFrame};

pub struct PidHandle(pub usize);

struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }
    pub fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            return PidHandle(pid);
        }
        let pid = self.current;
        self.current += 1;
        PidHandle(pid)
    }
    pub fn dealloc(&mut self, pid: usize) {
        assert!(pid < self.current);
        assert!(
            self.recycled.iter().find(|ppid| **ppid == pid).is_none(),
            "pid {} has been deallocated!", pid
        );
        self.recycled.push(pid);
    }
}

lazy_static! {
    static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> = unsafe {
        UPSafeCell::new(PidAllocator::new())
    };
}


impl Drop for PidHandle {
    fn drop(&mut self) {
        // 释放所有已分配的pid
        PID_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

pub fn alloc_pid() -> PidHandle {
    PID_ALLOCATOR.exclusive_access().alloc()
}

#[derive(Clone, Debug)]
pub struct KernelStack {
    frame:FrameTracker,
}

impl KernelStack {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        KernelStack {
            frame,
        }
    }
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

    pub fn get_trap_cx(&self) -> usize {
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

