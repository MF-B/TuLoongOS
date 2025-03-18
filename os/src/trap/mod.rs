pub mod context;
use core::arch::global_asm;

pub use context::TrapFrame;
use crate::batch::run_next_app;
use crate::syscall::syscall;
use loongArch64::register::estat::{self, Exception, Trap};
use loongArch64::register::{crmd, eentry, MemoryAccessType};
use log::*;

global_asm!(include_str!("trap.S"));

pub fn init() {
    unsafe extern "C" { fn __alltraps(); }
    // 设置中断入口点地址到 tcfg.tvec 寄存器
    set_exception_entry_base(__alltraps as usize);
}

#[inline]
pub fn set_exception_entry_base(eentry: usize) {
    eentry::set_eentry(eentry);
    crmd::set_datf(MemoryAccessType::StronglyOrderedUnCached);
    crmd::set_datm(MemoryAccessType::StronglyOrderedUnCached);
}

#[unsafe(no_mangle)]
fn trap_handler(tf: &mut TrapFrame) -> &mut TrapFrame {
    let estat = estat::read();

    match estat.cause() {
        Trap::Exception(Exception::Syscall) => {
            tf.era += 4;
            tf.regs.a0 = syscall(tf, tf.regs.a7) as usize;
        }
        Trap::Exception(Exception::StorePageFault) => {
            error!("StorePageFault in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::MemoryAccessAddressError) => {
            error!("MemoryAccessAddressError in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::InstructionNotExist) => {
            error!("InstructionNotExist in application, kernel killed it.");
            debug!(
                "trap {:?} @ {:#x}:\n",
                estat.cause(),
                tf.era,
                //tf
            );
            run_next_app();
        }
        // Trap::Exception(Exception::InstructionPrivilegeIllegal) => {
        //     error!("InstructionPrivilegeIllegal in application, kernel killed it.");
        //     run_next_app();
        // }
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
