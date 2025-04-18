use core::arch::asm;
use core::cell::RefMut;

use alloc::string::String;
use alloc::sync::Weak;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use log::info;
use loongArch64::register::pgdl;

use crate::config::PAGE_SIZE_BITS;
use crate::fs::stdio::Stdin;
use crate::fs::stdio::Stdout;
use crate::fs::File;
use crate::mm::translated_refmut;
use crate::sync::UPSafeCell;
use crate::mm::memory_set::MemorySet;
use crate::trap::TrapFrame;

use super::signal::SignalActions;
use super::signal::SignalFlags;
use super::{pid::{alloc_pid, KernelStack, PidHandle}, task::{TaskContext, TaskStatus}};



pub struct ProcessControlBlock {
    // 不可变部分immutable
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    // mutable
    inner: UPSafeCell<ProcessControlBlockInner>,
}

pub struct ProcessControlBlockInner {
    // 信号量
    pub handling_sig: isize,
    pub trap_ctx_backup: Option<TrapFrame>,
    pub signals: SignalFlags,
    pub signal_mask: SignalFlags,
    pub signal_actions: SignalActions,
    // 状态
    pub killed: bool,
    pub frozen: bool,

    pub base_size: usize,
    pub task_context: TaskContext,
    pub task_status: TaskStatus,
    pub memory_set: MemorySet,

    pub parent: Option<Weak<ProcessControlBlock>>,
    pub children: Vec<Arc<ProcessControlBlock>>,

    pub exit_code: i32,
    pub fd_table: Vec<Option<Arc<dyn File + Send + Sync>>>,
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
                handling_sig: -1,
                trap_ctx_backup: None,
                signals: SignalFlags::empty(),
                signal_mask: SignalFlags::empty(),
                signal_actions: SignalActions::default(),
                killed: false,
                frozen: false,
                base_size: user_sp,
                task_context,
                task_status,
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
            }) }, 
        }
    }

    pub fn get_trap_cx(&self) -> &'static mut TrapFrame {
        self.kernel_stack.get_mut::<TrapFrame>()
    }

    pub fn inner_exclusive_access(&self) -> RefMut<'_, ProcessControlBlockInner> {
        self.inner.exclusive_access()
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
        let (memory_set, mut user_sp, entry_point) = MemorySet::from_elf(elf_data);

        // push arguments on user stack
        user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
        let argv_base = user_sp;
        let mut argv: Vec<_> = (0..=args.len())
            .map(|arg| {
                translated_refmut(
                    memory_set.token(),
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
                *translated_refmut(memory_set.token(), p as *mut u8) = *c;
                p += 1;
            }
            *translated_refmut(memory_set.token(), p as *mut u8) = 0;
        }
        // make the user_sp aligned to 8B for k210 platform
        user_sp -= user_sp % core::mem::size_of::<usize>();

        let mut inner = self.inner_exclusive_access();
        inner.memory_set = memory_set;
        let mut trap_cx = TrapFrame::app_init_context(entry_point, user_sp);
        trap_cx.regs.a0 = args.len();
        trap_cx.regs.a1 = argv_base;
        self.kernel_stack.push_context(trap_cx);
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

        let c_base_size = parent_pcb.base_size;
        let c_task_context = TaskContext::goto_trap_return(c_kernel_stack.get_trap_cx());
        let c_memory_set = MemorySet::from_existed_process(&parent_pcb.memory_set);

        let mut new_fd_table: Vec<Option<Arc<dyn File + Send + Sync>>> = Vec::new();
        for fd in parent_pcb.fd_table.iter() {
            if let Some(file) = fd {
                new_fd_table.push(Some(file.clone()));
            } else {
                new_fd_table.push(None);
            }
        }
        let child_pcb = Arc::new(ProcessControlBlock {
            pid,
            kernel_stack: c_kernel_stack,
            inner: unsafe { UPSafeCell::new(ProcessControlBlockInner {
                handling_sig: parent_pcb.handling_sig,
                trap_ctx_backup: parent_pcb.trap_ctx_backup.clone(),
                signals: parent_pcb.signals,
                signal_mask: parent_pcb.signal_mask,
                signal_actions: parent_pcb.signal_actions.clone(),
                killed: parent_pcb.killed,
                frozen: parent_pcb.frozen,
                base_size: c_base_size,
                task_context: c_task_context,
                task_status: TaskStatus::Ready,
                memory_set: c_memory_set,
                parent: Some(Arc::downgrade(self)),
                children: Vec::new(),
                exit_code: 0,
                fd_table: new_fd_table,
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
    pub fn alloc_fd(&mut self) -> usize {
        if let Some(fd) = (0..self.fd_table.len()).find(|fd| self.fd_table[*fd].is_none()) {
            fd
        } else {
            self.fd_table.push(None);
            self.fd_table.len() - 1
        }
    }
}


