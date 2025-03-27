pub mod context;
use core::arch::{asm, global_asm};
pub use context::TrapFrame;
use loongArch64::register::ecfg::LineBasedInterrupt;
use crate::mm::{PageTable,VirtAddr, VirtPageNum};
use crate::syscall::syscall;
use crate::task::{current_trap_cx, current_user_token, exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use loongArch64::register::estat::{self, Exception, Interrupt, Trap};
use loongArch64::register::{crmd, dmw0, ecfg, eentry, pwch, pwcl, stlbps, tcfg, ticlr, tlbelo0, tlbelo1, tlbidx, tlbrbadv, tlbrehi, tlbrelo0, tlbrelo1, tlbrentry};
use log::*;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("tlb.S"));

pub fn init() {
//unsafe extern "C" { fn __alltraps(); }
    // 设置中断入口点地址到 tcfg.tvec 寄存器

    // 为内核设置直接映射地址翻译模式
    dmw0::set_vseg(0x0);
    dmw0::set_plv0(true);
    dmw0::set_plv3(false);
    //dmw0::set_mat(MemoryAccessType::StronglyOrderedUnCached);

    unsafe extern "C" {
        fn __alltraps();
        fn __tlb_rfill();
        //fn kernel_trap_entry();
    }

    set_exception_entry_base(__alltraps as usize);
    tlbrentry::set_tlbrentry(__tlb_rfill as usize);
    stlbps::set_ps(0xe);
    tlbrehi::set_ps(0xe); //设置STLB的页面大小为16KiB
    pwcl::set_pte_width(8);
    pwcl::set_ptbase(0xe);
    pwcl::set_ptwidth(0xb);
    pwcl::set_dir1_base(25); //页目录表起始位置
    pwcl::set_dir1_width(0xb); //页目录表宽度为11位

    pwch::set_dir3_base(36); //第三级页目录表
    pwch::set_dir3_width(0xb); //页目录表宽度为11位
        // 开启
    crmd::set_pg(true);
    crmd::set_da(false);

    enable_timer_interrupt();

    unsafe {
        asm!("invtlb 0,$r0,$r0"); //清除TLB
    }
}

#[inline]
pub fn set_exception_entry_base(eentry: usize) {
    eentry::set_eentry(eentry);
    //crmd::set_datf(MemoryAccessType::CoherentCached);
    //crmd::set_datm(MemoryAccessType::CoherentCached);
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
            // trace!(
            //     "trap {:?} @ {:#x}:\n{:#x?}",
            //     estat.cause(),
            //     tf.era,
            //     tf
            // );
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
                "trap {:?} @ {:#x}:\n{:#x?}\n{:?}",
                estat.cause(),
                tf.era,
                tf,
                crmd
            );
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::InstructionPrivilegeIllegal) => {
            error!("InstructionPrivilegeIllegal in application, kernel killed it.");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::TLBRFill) => {
            tlb_refill_handler()
            //suspend_current_and_run_next();
        }
        Trap::Exception(Exception::PageModifyFault) => {
            tlb_page_modify_handler();
            //exit_current_and_run_next();
        }
        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}\necode: {:#x?}",
                estat.cause(),
                tf.era,
                tf,
                estat.ecode()
            );
        }
    }
    tf
}

fn tlb_refill_handler() {
    unsafe { asm!(
                "csrrd $t0, 0x1B",
                "lddir $t0, $t0, 3",
                "lddir $t0, $t0, 1",
                "ldpte $t0, 0",
                "ldpte $t0, 1",
                )
            };
    //println!("find ppn0: {:#x} ppn1: {:#x} for vpn: {:#x}",ppn0 >> 14,ppn1>>14,vppn>>1);

    tlbrelo0::set_dirty(true);
    tlbrelo1::set_dirty(true);

    unsafe {
        asm!("tlbfill");
    }
}

pub fn set_user_trap_entry() {
    // 初始化
    unsafe extern "C" {
        fn __alltraps();
    }
    eentry::set_eentry(__alltraps as usize); //设置普通异常和中断入口
}

#[unsafe(no_mangle)]
pub fn trap_return() {
    set_user_trap_entry();
    let trap_cx = current_trap_cx();
    unsafe extern "C" {
        fn __restore();
    }
    unsafe {
        asm!("move $a0,{}",in(reg)trap_cx);
        __restore();
    }
}

fn tlb_page_modify_handler() {
    //找到对应的页表项，修改D位为1
    let badv = tlbrbadv::read(); //出错虚拟地址
    let vpn: VirtAddr = badv.vaddr().into(); //虚拟地址
    let vpn: VirtPageNum = vpn.floor(); //虚拟地址的虚拟页号
    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let pte = page_table.find_pte(vpn).unwrap(); //获取页表项
    pte.set_dirty(); //修改D位为1
    unsafe {
        asm!("tlbsrch", "tlbrd",); //根据TLBEHI的虚双页号查询TLB对应项
    }
    let tlbidx = tlbidx::read(); //获取TLB项索引
    assert_eq!(tlbidx.ne(), false);
    tlbelo0::set_dirty(true);
    tlbelo1::set_dirty(true);
    unsafe {
        asm!("tlbwr"); //重新将tlbelo写入tlb
    }
}