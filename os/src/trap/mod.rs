pub mod context;
use core::arch::global_asm;

pub use context::TrapFrame;
use loongArch64::register::ecfg::LineBasedInterrupt;
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use loongArch64::register::estat::{self, Exception, Interrupt, Trap};
use loongArch64::register::{crmd, ecfg, eentry, tcfg, ticlr, MemoryAccessType};
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

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    ticlr::clear_timer_interrupt();
    set_next_trigger();
    tcfg::set_en(true);
    tcfg::set_periodic(true);

    ecfg::set_lie(LineBasedInterrupt::TIMER);
    ecfg::set_vs(0);

    crmd::set_ie(false);
    info!("interrupt enable: {:?}", ecfg::read().lie());
}

#[unsafe(no_mangle)]
fn trap_handler(tf: &mut TrapFrame) -> &mut TrapFrame {
    let estat = estat::read();
    let crmd = crmd::read();
    if crmd.ie() {
        // 全局中断会在中断处理程序被关掉
        panic!("kerneltrap: global interrupt enable");
    }

    match estat.cause() {
        Trap::Interrupt(Interrupt::Timer) => {
            trace!("timer interrupt from user");
            ticlr::clear_timer_interrupt();
            suspend_current_and_run_next();
        }
        Trap::Exception(Exception::Syscall) => {
            tf.era += 4;
            tf.regs.a0 = syscall(tf, tf.regs.a7) as usize;
        }
        Trap::Exception(Exception::StorePageFault) => {
            error!("StorePageFault in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::MemoryAccessAddressError) => {
            error!("MemoryAccessAddressError in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::InstructionNotExist) => {
            error!("InstructionNotExist in application, kernel killed it.");
            debug!(
                "trap {:?} @ {:#x}:\n{:#x?}",
                estat.cause(),
                tf.era,
                tf
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::InstructionPrivilegeIllegal) => {
            error!("InstructionPrivilegeIllegal in application, kernel killed it.");
            exit_current_and_run_next();
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
