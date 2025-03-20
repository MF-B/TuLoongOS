use core::arch::asm;

use log::info;

use crate::config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT, KERNEL_STACK_SIZE, MAX_APP_NUM, USER_STACK_SIZE};
use crate::trap::TrapFrame;

#[derive(Copy,Clone)]
pub struct KernelStack{
    data: [usize;KERNEL_STACK_SIZE]
}

#[derive(Copy,Clone)]
struct UserStack{
    data: [usize;USER_STACK_SIZE]
}

impl KernelStack{
    fn push_context(&self, cx: TrapFrame)-> usize{
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
        unsafe { *cx_ptr = cx };
        cx_ptr as usize
        //unsafe { cx_ptr.as_mut().unwrap() }
    }
    fn get_sp(&self) -> usize{
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
}
impl UserStack {
    fn get_sp(&self) -> usize{
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub static KERNEL_STACK: [KernelStack;MAX_APP_NUM] = [KernelStack{data: [0;KERNEL_STACK_SIZE]};MAX_APP_NUM];
static USER_STACK: [UserStack;MAX_APP_NUM] = [UserStack{data: [0;USER_STACK_SIZE]};MAX_APP_NUM];

pub fn init_app_cx(app_id: usize) -> usize{
    KERNEL_STACK[app_id].push_context(
        TrapFrame::app_init_context(get_base_i(app_id), USER_STACK[app_id].get_sp()),
    )
}

pub fn load_app() {
    unsafe extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    // 获取各个app的起始地址数组并返回给APP_MAMAGER
    let app_start = unsafe {
        core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
    };
    // load apps
    for i in 0..num_app {
        let base_i = get_base_i(i);
        // clear region
        (base_i..base_i + APP_SIZE_LIMIT)
            .for_each(|addr| unsafe { (addr as *mut u8).write_volatile(0) });
        // load app from data section to memory
        let src = unsafe {
            core::slice::from_raw_parts(app_start[i] as *const u8, app_start[i + 1] - app_start[i])
        };
        info!("load app{} from {:#x} to {:#x}", i, app_start[i], base_i);
        let dst = unsafe { core::slice::from_raw_parts_mut(base_i as *mut u8, src.len()) };
        dst.copy_from_slice(src);
    }
    unsafe {
        asm!("ibar 0"); // 指令缓存刷新
    }
}

pub fn get_base_i(app_id: usize) -> usize {
    APP_BASE_ADDRESS + app_id * APP_SIZE_LIMIT
}

pub fn get_num_app() -> usize {
    unsafe {
        unsafe extern "C" {
            fn _num_app();
        }
        let num_app_ptr = _num_app as usize as *const usize;
        num_app_ptr.read_volatile()
    }
}

// pub fn get_app_start() -> &'static [usize] {
//     unsafe extern "C" {
//         fn _num_app();
//     }
//     let num_app_ptr = _num_app as usize as *const usize;
//     let num_app = get_num_app();
//     let app_start =
//         unsafe { core::slice::from_raw_parts(num_app_ptr.add(1) as *const usize, num_app + 1) };
//     app_start
// }
