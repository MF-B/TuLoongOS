pub mod context;
use core::arch::global_asm;

pub use context::TrapFrame;
use crate::batch::run_next_app;
use crate::syscall::syscall;
use loongArch64::register::estat::{self, Exception, Trap};
use loongArch64::register::{ecfg, eentry};
use log::*;

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe extern "C" { fn __alltraps(); }
    // 设置中断入口点地址到 tcfg.tvec 寄存器
    set_exception_entry_base(__alltraps as usize);
}

#[inline]
pub fn set_exception_entry_base(eentry: usize) {
    ecfg::set_vs(0);
    eentry::set_eentry(eentry);
}

#[unsafe(no_mangle)]
fn trap_handler(tf: &mut TrapFrame) -> &mut TrapFrame {
    let estat = estat::read();

    match estat.cause() {
        Trap::Exception(Exception::Syscall) => {
            debug!(
                "trap {:?} @ {:#x}:\n{:#x?}",
                estat.cause(),
                tf.era,
                tf
            );
            tf.era += 4;
            tf.regs.a0 = syscall(tf, tf.regs.a7) as usize;
        }
        Trap::Exception(Exception::StorePageFault) => {
            error!("StorePageFault in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::InstructionPrivilegeIllegal) => {
            error!("InstructionPrivilegeIllegal in application, kernel killed it.");
            run_next_app();
        }
        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                estat.cause(),
                tf.era,
                tf
            );
        }
    }
    tf
}
