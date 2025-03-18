use crate::config::{KERNEL_STACK_SIZE, MAX_APP_NUM, USER_STACK_SIZE};
use crate::loader::{get_app_start, get_base_i, get_num_app, load_app};
use crate::misc::terminate;
use crate::sync::UPSafeCell;
use crate::trap::TrapFrame;

use lazy_static::*;
use log::*;


#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
#[derive(Copy,Clone)]
pub struct UserStack {
    pub data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: KernelStack = KernelStack { data: [0; KERNEL_STACK_SIZE] };
pub static USER_STACK: [UserStack; MAX_APP_NUM] = [UserStack { data: [0; USER_STACK_SIZE] }; MAX_APP_NUM];

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    pub fn push_context(&self, cx: TrapFrame) -> &'static mut TrapFrame {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
        unsafe { *cx_ptr = cx };
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}
impl UserStack {
    pub fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

struct AppManager {
    num_app: usize,
    current_app: usize,
}


impl AppManager {
    pub fn print_app_info(&self) {
        let app_start = get_app_start();
        info!("num_app = {}", self.num_app);
        for i in 0..self.num_app {
            info!(
                "app_{} [{:#x}, {:#x})",
                i,
                app_start[i],
                app_start[i + 1]
            );
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}


// 定义全局变量
lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe { 
        UPSafeCell::new({
            // 获取app的数量
            let num_app = get_num_app();
            // 设置返回值
            AppManager {
                num_app,
                current_app: 0,
            }
        })
    };
}

// 初始化批处理系统
pub fn init() {
    print_app_info();
    load_app();
}

pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

pub fn run_next_app() -> ! {
    // 引用app_manager,加载程序
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    if current_app >= get_num_app() {
        info!("No more apps to run!");
        terminate();
    }
    info!("Loading app_{}", current_app);
    debug!("Entry: {:#x}", get_base_i(current_app));
    debug!("Stack: {:#x}", USER_STACK[current_app].get_sp());
    app_manager.move_to_next_app();
    // 释放app_manager
    drop(app_manager);
    // 准备并跳转到用户应用
    unsafe extern "C" {
        pub fn __restore(cx_addr: usize);
    }
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapFrame::app_init_context(
            get_base_i(current_app),USER_STACK[current_app].get_sp()
        )) as *mut TrapFrame as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}