use crate::config::{KERNEL_STACK_SIZE, MAX_APP_NUM};
use crate::trap::TrapFrame;

#[derive(Copy, Clone)]
pub struct KernelStack {
    data: [usize; KERNEL_STACK_SIZE],
}
impl KernelStack {
    pub fn push_context(&self, trap_cx: TrapFrame) -> usize {
        let trap_cx_ptr = (self.get_sp() - core::mem::size_of::<TrapFrame>()) as *mut TrapFrame;
        unsafe {
            *trap_cx_ptr = trap_cx;
        }
        trap_cx_ptr as usize
    }
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn get_trap_cx(&self) -> usize {
        self.get_sp() - core::mem::size_of::<TrapFrame>()
    }
}

pub static KERNEL_STACK: [KernelStack; MAX_APP_NUM] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_APP_NUM];

pub fn init_app_cx(app_id: usize, entry_point: usize, user_sp: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapFrame::app_init_context(entry_point, user_sp))
}

pub fn get_app_trap_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].get_trap_cx()
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

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    unsafe extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start = unsafe { core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1) };
    assert!(app_id < num_app);
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id],
        )
    }
}
