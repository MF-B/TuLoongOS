use core::arch::asm;
use core::cell::RefMut;

use alloc::string::String;
use alloc::sync::Weak;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use loongArch64::register::pgdl;

use crate::config::PAGE_SIZE_BITS;
use crate::fs::stdio::Stdin;
use crate::fs::stdio::Stdout;
use crate::fs::File;
use crate::mm::translated_refmut;
use crate::sync::mutex::Mutex;
use crate::sync::UPSafeCell;
use crate::mm::memory_set::MemorySet;
use crate::trap::TrapFrame;

use super::id::RecycleAllocator;
use super::manager::add_task;
use super::manager::insert_into_pid2process;
use super::signal::SignalFlags;
use super::task::TaskControlBlock;
use super::id::{pid_alloc, PidHandle};



pub struct ProcessControlBlock {
    // 不可变部分immutable
    pub pid: PidHandle,
    // mutable
    inner: UPSafeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    // 信号
    pub signals: SignalFlags,
    // 状态
    pub is_zombie: bool,

    pub memory_set: MemorySet,

    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,

    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,

    // 线程控制器
    pub tasks: Vec<Option<Arc<TaskControlBlock>>>,
    pub task_res_allocator: RecycleAllocator,
    pub mutex_list: Vec<Option<Arc<dyn Mutex>>>,
//    pub semaphore_list: Vec<Option<Arc<Semaphore>>>,
//    pub condvar_list: Vec<Option<Arc<Condvar>>>,
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        let pid_handle = pid_alloc();

        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);

        // create PCB
        let process = Arc::new(Self {
            pid: pid_handle,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
//                    semaphore_list: Vec::new(),
//                    condvar_list: Vec::new(),
                })
            },
        });

        let task = Arc::new(TaskControlBlock::new(
            Arc::clone(&process),
            ustack_base,
            true,
        ));

        let task_inner = task.inner_exclusive_access();
        let trap_cx = task_inner.kstack.get_mut::<TrapFrame>();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        task_inner.kstack.init_app_cx(entry_point, ustack_top);
        drop(task_inner);

        *trap_cx = TrapFrame::app_init_context(entry_point, ustack_top);

        // add main thread to the process
        let mut process_inner = process.inner_exclusive_access();
        process_inner.tasks.push(Some(Arc::clone(&task)));
        drop(process_inner);
        insert_into_pid2process(process.getpid(), Arc::clone(&process));
        // add main thread to scheduler
        // debug!("task: ustack_base={:#x}, trap_cx={:#x?}", ustack_base, trap_cx);
        add_task(task);
        process
    }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        assert_eq!(self.inner_exclusive_access().thread_count(), 1);
        let (memory_set, ustack_base, entry_point) = MemorySet::from_elf(elf_data);
        let new_token = memory_set.token();

        self.inner_exclusive_access().memory_set = memory_set;

        let task = self.inner_exclusive_access().get_task(0); //得到主进程的内容
        let mut task_inner = task.inner_exclusive_access();
        task_inner.res.as_mut().unwrap().ustack_base = ustack_base;
        task_inner.res.as_mut().unwrap().alloc_user_res(); //重新分配资源

        // push arguments on user stack
        let mut user_sp = task_inner.res.as_mut().unwrap().ustack_top();
        user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_sp;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| {
                translated_refmut(
                    new_token,
                    (argv_base + arg * core::mem::size_of::<usize>()) as *mut usize
                )
            })
            .collect();
        *argv[args.len()] = 0;
        for i in 0..args.len() {
            user_sp -= args[i].len() + 1;
            *argv[i] = user_sp;
            let mut p = user_sp;
            for c in args[i].as_bytes() {
                *translated_refmut(new_token, p as *mut u8) = *c;
                p += 1;
            }
            *translated_refmut(new_token, p as *mut u8) = 0;
        }
        // make the user_sp aligned to 8B for k210 platform
        user_sp -= user_sp % core::mem::size_of::<usize>();

        let mut trap_cx = TrapFrame::app_init_context(entry_point, user_sp);
        trap_cx.regs.a0 = args.len();
        trap_cx.regs.a1 = argv_base;
        let pid = self.getpid();
        *task_inner.kstack.get_mut::<TrapFrame>() = trap_cx;
        unsafe {
            asm!("invtlb 0x4,{},$r0",in(reg) pid);
        }
        let pgd = new_token << PAGE_SIZE_BITS;
        pgdl::set_base(pgd);
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut parent = self.inner_exclusive_access();
        assert_eq!(parent.thread_count(), 1);
        // clone parent's memory_set completely including trampoline/ustacks/trap_cxs
        let memory_set = MemorySet::from_existed_process(&parent.memory_set);
        // alloc a pid
        let pid = pid_alloc();
        // copy fd table
        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(Arc::clone(file)));
            } else {
                new_fd_table.push(None);
            }
        }
        // create child process pcb
        let child = Arc::new(Self {
            pid,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    is_zombie: false,
                    memory_set,
                    parent: None,
                    children: Vec::new(),
                    exit_code: 0,
                    fd_table: vec![
                        // 0 -> stdin
                        Some(Arc::new(Stdin)),
                        // 1 -> stdout
                        Some(Arc::new(Stdout)),
                        // 2 -> stderr
                        Some(Arc::new(Stdout)),
                    ],
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_res_allocator: RecycleAllocator::new(),
                    mutex_list: Vec::new(),
//                    semaphore_list: Vec::new(),
//                    condvar_list: Vec::new(),
                })
            },
        });

        // add child
        parent.children.push(Arc::clone(&child));
        // create main thread of child process
        let task = Arc::new(TaskControlBlock::new(
            Arc::clone(&child),
            parent
                .get_task(0)
                .inner_exclusive_access()
                .res
                .as_ref()
                .unwrap()
                .ustack_base(),
            // here we do not allocate trap_cx or ustack again
            // but mention that we allocate a new kstack here
            false,
        ));
        // attach task to child process
        let mut child_inner = child.inner_exclusive_access();
        child_inner.tasks.push(Some(Arc::clone(&task)));
        drop(child_inner);
        // modify kstack_top in trap_cx of this thread
        let task_inner = task.inner_exclusive_access();
        // 修改trap_cx的内容，使其保持与父进程相同
        // 这需要拷贝父进程的主线程的内核栈到子进程的内核栈中
        task_inner
            .kstack
            .copy_from_existed(&parent.get_task(0).inner_exclusive_access().kstack);
        drop(task_inner);

        insert_into_pid2process(child.getpid(), Arc::clone(&child));
        // add this thread to scheduler
        add_task(task);
        child
    }
}

impl ProcessControlBlockInner {
    #[allow(unused)]
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn is_zombie(&self) -> bool {
        self.is_zombie
    }
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
    pub fn alloc_tid(&mut self) -> usize {
        self.task_res_allocator.alloc()
    }
    pub fn dealloc_tid(&mut self, id: usize) {
        self.task_res_allocator.dealloc(id);
    }
    pub fn thread_count(&self) -> usize {
        self.tasks.len()
    }
    pub fn get_task(&self, id: usize) -> Arc<TaskControlBlock> {
        self.tasks[id].as_ref().unwrap().clone()
    }
}


