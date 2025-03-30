pub mod context;
use core::arch::{asm, global_asm};
pub use context::TrapFrame;
use loongArch64::register::ecfg::LineBasedInterrupt;
use crate::mm::{PageTable,VirtAddr, VirtPageNum};
use crate::syscall::syscall;
use crate::task::processor::{current_trap_cx, current_user_token};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use loongArch64::register::estat::{self, Exception, Interrupt, Trap};
use loongArch64::register::{badv, crmd, dmw0, ecfg, eentry, pgd, pwch, pwcl, stlbps, tcfg, ticlr, tlbelo0, tlbelo1, tlbidx, tlbrbadv, tlbrehi, tlbrelo0, tlbrelo1, tlbrentry};
use log::*;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("tlb.S"));

pub fn init() {
    // 为内核设置直接映射地址翻译模式
    dmw0::set_vseg(0x0);
    dmw0::set_plv0(true);
    dmw0::set_plv3(false);
    unsafe extern "C" {
        fn __alltraps();
        fn __tlb_rfill();
    }

    set_exception_entry_base(__alltraps as usize);
    tlbrentry::set_tlbrentry(__tlb_rfill as usize);
    //tlbrentry::set_tlbrentry(__alltraps as usize);
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
        Trap::Exception(Exception::StorePageFault) | Trap::Exception(Exception::LoadPageFault) |
        Trap::Exception(Exception::MemoryAccessAddressError) | Trap::Exception(Exception::InstructionNotExist) |
        Trap::Exception(Exception::FetchPageFault)
         => {
            // error!(
            //     "Unhandled trap {:?} @ {:#x}:\n{:#x?}\necode: {:#x?}",
            //     estat.cause(),
            //     tf.era,
            //     tf,
            //     estat.ecode()
            // );
            error!("{:?} in process, kernel killed it.", estat.cause());
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::InstructionPrivilegeIllegal) => {
            error!("InstructionPrivilegeIllegal in application, kernel killed it.");
            exit_current_and_run_next(-3);
        }
        Trap::Exception(Exception::TLBRFill) => {
            tlb_refill_handler();
        }
        Trap::Exception(Exception::PageModifyFault) => {
            tlb_page_modify_handler();
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

#[unsafe(no_mangle)]
fn tlb_page_fault(){
    //检查pagefault相关内容
    // unsafe {
    //     asm!(
    //         "tlbsrch",
    //         "tlbrd",
    //     )
    // }
    let badv = badv::read().vaddr();
    let token = current_user_token();
    let vpn: VirtAddr = badv.into(); //虚拟地址
    let vpn: VirtPageNum = vpn.floor(); //虚拟地址的虚拟页号
    let page_table = PageTable::from_token(token);
    error!("badv: {:#x}", badv);
    error!("pgd: {:#x}", pgd::read().base());
    error!("crmd: {:?}", crmd::read().plv());
    error!("vpn: {:x?}", vpn);
    error!("token: {:#x}", token);
    error!("page_table: {:#x}", page_table.token());
    if let Some(pte) = page_table.find_pte(vpn){
        info!("badv:{:#x} has pte:{:?}",badv,pte);
    }else{
        info!("badv:{:#x} hasn't pte",badv);
    }

}