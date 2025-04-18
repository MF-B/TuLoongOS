use alloc::string::String;
use alloc::vec::*;
use alloc::{fmt, vec};
use bit_field::BitField;
use bitflags::*;

use crate::config::{LEVELS, PAGE_SIZE_BITS, PALEN};

use super::{
    FrameTracker,
    address::{PhysPageNum, VirtPageNum},
    frame_alloc,
};
use super::{PhysAddr, StepByOne, VirtAddr};

bitflags! {
    pub struct PTEFlags: usize {
        const V = 1 << 0;
        const D = 1 << 1;
        const PLV0 = 1 << 2;
        const PLV1 = 1 << 3;
        const MAT0 = 1 << 4;
        const MAT1 = 1 << 5;
        const G = 1 << 6;
        const P = 1 << 7;
        const W = 1 << 8;

        const NR = 1 << 61;
        const NX = 1 << 62;
        const RPLV = 1 << 63;
    }
}

#[derive(Clone, Copy)]
pub struct PageTableEntry {
    bits: usize,
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PageTableEntry RPLV:{},NX:{},NR:{},PPN:{:#x},W:{},P:{},G:{},MAT:{},PLV:{},D:{},V:{}",
            self.bits.get_bit(63),
            self.bits.get_bit(62),
            self.bits.get_bit(61),
            self.bits.get_bits(14..PALEN),
            self.bits.get_bit(8),
            self.bits.get_bit(7),
            self.bits.get_bit(6),
            self.bits.get_bits(4..=5),
            self.bits.get_bits(2..=3),
            self.bits.get_bit(1),
            self.bits.get_bit(0)
        )
    }
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        let mut bits = 0usize;
        bits.set_bits(14..PALEN, ppn.0);
        bits = bits | flags.bits();
        PageTableEntry { bits }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> PhysPageNum {
        self.bits.get_bits(14..PALEN).into()
    }
    // 返回物理页号---页目录项
    // 在一级和二级页目录表中目录项存放的是只有下一级的基地址
    pub fn directory_ppn(&self) -> PhysPageNum {
        (self.bits >> PAGE_SIZE_BITS).into()
    }
    pub fn flags(&self) -> PTEFlags {
        //这里只需要标志位，需要把非标志位的位置清零
        let mut bits = self.bits;
        bits.set_bits(14..PALEN, 0);
        PTEFlags::from_bits(bits).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        !((self.flags() & PTEFlags::NR) != PTEFlags::empty())
    }
    pub fn executable(&self) -> bool {
        !((self.flags() & PTEFlags::NX) != PTEFlags::empty())
    }
    pub fn set_dirty(&mut self) {
        self.bits.set_bit(1, true);
    }
    pub fn is_zero(&self) -> bool {
        self.bits == 0
    }
}

#[derive(Default, Clone)]
pub struct PageTable {
    root_ppn: PhysPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        //println!("find_pte_create vpn {:?} idxs {:?}", vpn, idxs);
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..LEVELS {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if i == LEVELS - 1 {
                result = Some(pte);
                break;
            }
            if pte.is_zero() {
                let frame = frame_alloc().unwrap();
                // 页目录项只保存地址
                *pte = PageTableEntry {
                    bits: frame.ppn.0 << PAGE_SIZE_BITS,
                };
                self.frames.push(frame);
            }
            ppn = pte.directory_ppn();
        }
        result
    }
    pub fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for i in 0..LEVELS {
            let pte = &mut ppn.get_pte_array()[idxs[i]];
            if pte.is_zero() {
                return None;
            }
            if i == LEVELS - 1 {
                result = Some(pte);
                break;
            }
            ppn = pte.directory_ppn();
        }
        result
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub fn translate_va(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            //println!("translate_va:va = {:?}", va);
            let aligned_pa: PhysAddr = pte.ppn().into();
            //println!("translate_va:pa_align = {:?}", aligned_pa);
            let offset = va.page_offset();
            let aligned_pa_usize: usize = aligned_pa.into();
            (aligned_pa_usize + offset).into()
        })
    }

    pub fn token(&self) -> usize {
        self.root_ppn.0
    }
    pub fn from_token(pgd: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(pgd & ((1usize << 34) - 1)),
            frames: Vec::new(),
        }
    }
}

pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into();
    }
    v
}

pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table.translate_va(VirtAddr::from(va)).unwrap().get_mut());
        if ch == 0 {
            break;
        } else {
            string.push(ch as char);
            va += 1;
        }
    }
    string
}  

pub fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T {
    let page_table = PageTable::from_token(token);
    page_table
        .translate_va(VirtAddr::from(ptr as usize))
        .unwrap()
        .get_ref()
}

///translate a generic through page table and return a mutable reference
pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    //println!("into translated_refmut!");
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    //println!("translated_refmut: before translate_va");
    page_table
        .translate_va(VirtAddr::from(va))
        .unwrap()
        .get_mut()
}

pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}
impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}
/// Iterator of `UserBuffer`
pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}