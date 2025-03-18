use crate::config::{APP_BASE_ADDRESS, APP_SIZE_LIMIT};

pub fn load_app() {
    unsafe {
        unsafe extern "C" {
            fn _num_app();
        }
        let num_app_ptr = _num_app as usize as *const usize;
        let num_app = get_num_app();
        // 获取各个app的起始地址数组并返回给APP_MAMAGER
        let app_start =
            core::slice::from_raw_parts(num_app_ptr.add(1) as *const usize, num_app + 1);
        for i in 0..num_app {
            core::slice::from_raw_parts_mut(get_base_i(i) as *mut u8, APP_SIZE_LIMIT).fill(0);
            // 复制app的代码到运行app的内存区域
            let app_src = core::slice::from_raw_parts(
                app_start[i] as *const u8,
                app_start[i + 1] - app_start[i],
            );
            let app_dst =
                core::slice::from_raw_parts_mut(get_base_i(i) as *mut u8, app_src.len());
            app_dst.copy_from_slice(app_src);
            // memory fence about fetching the instruction memory
            // asm!("dbar 0");
        }
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

pub fn get_app_start() -> &'static [usize] {
    unsafe extern "C" {
        fn _num_app();
    }
    let num_app_ptr = _num_app as usize as *const usize;
    let num_app = get_num_app();
    let app_start =
    unsafe { core::slice::from_raw_parts(num_app_ptr.add(1) as *const usize, num_app + 1) };
    app_start
}
