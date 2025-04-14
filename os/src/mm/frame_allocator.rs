use alloc::vec::Vec;
use crate::sync::UPSafeCell;
use crate::config::{MEMORY_LOW_END, MEMORY_HIGH_START, MEMORY_HIGH_END};
use lazy_static::*;
use super::address::{PhysAddr, PhysPageNum};

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

pub struct StackFrameAllocator {
    current: usize, 
    end_low: usize,      //低地址段空闲内存的结束物理页号
    end_high: usize,     //高地址段空闲内存的结束物理页号
    recycled: Vec<usize>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            current: 0,
            end_low: 0,
            end_high: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        // 优先使用回收的页帧
        if let Some(addr) = self.recycled.pop() {
            return Some(PhysPageNum(addr));
        }
        // 优先从低地址段分配
        if self.current < self.end_low {
            let ppn = PhysPageNum(self.current);
            self.current += 1;
            return Some(ppn);
        }
        if self.current == self.end_low {
            self.current = MEMORY_HIGH_START;
        }
        // 如果低地址段已满，从高地址段分配
        else if self.current < self.end_high {
            let ppn = PhysPageNum(self.current);
            self.current += 1;
            return Some(ppn);
        }
        None
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        // 检查页帧是否在已分配的范围内
        let is_in_low_range = ppn.0 < self.current && ppn.0 < self.end_low;
        let is_in_high_range = ppn.0 < self.current && ppn.0 >= MEMORY_HIGH_START && ppn.0 < self.end_high;
        
        if (!is_in_low_range && !is_in_high_range) || self.recycled.contains(&ppn.0) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn.0);
        }
        self.recycled.push(ppn.0);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, start: PhysPageNum, end_low: PhysPageNum, end_high: PhysPageNum) {
        self.current = start.0;
        self.end_low = end_low.0;
        self.end_high = end_high.0;
    }
}

// 管理器在内核中只需要一个,故设置成全局实例

type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}

#[derive(Debug,Clone,Default)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

// 功能函数
pub fn init_frame_allocator() {
    unsafe extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_LOW_END).floor(),
        PhysAddr::from(MEMORY_HIGH_END).floor(),
    );
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.exclusive_access().alloc().map(|ppn|FrameTracker::new(ppn))
}

pub fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame: FrameTracker = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}