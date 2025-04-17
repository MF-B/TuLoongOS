use alloc::sync::Arc;
use log::info;
use core::mem::transmute;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use easy_fs::BlockDevice;
use core::cell::UnsafeCell;
use virtio_drivers::{
    device::blk::VirtIOBlk, transport::pci::PciTransport, BufferDirection, Hal, PhysAddr as VirtioPhysAddr
};

use crate::config::PAGE_SIZE;
use crate::drivers::pci::pci::init_virtio_blk;
use crate::mm::{frame_alloc, frame_dealloc, PhysAddr};
use crate::sync::UPSafeCell;

// lazy_static! {
//     pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = {
//         match init_virtio_blk() {
//             Some(block_device) => Arc::new(block_device),
//             None => panic!("Failed to find VirtIO block device"),
//         }
//     };
// }
/// Used only for initialization hacks.
pub const DUMMY_BLOCK_DEVICE: *const dyn BlockDevice =
    unsafe { transmute(&0 as *const _ as *const VirtIOBlock as *const dyn BlockDevice) };

pub static BLOCK_DEVICE: Cell<Arc<dyn BlockDevice>> = unsafe { transmute(DUMMY_BLOCK_DEVICE) };

pub fn ahci_init() {
    unsafe {
        (BLOCK_DEVICE.get() as *mut Arc<dyn BlockDevice>).write(Arc::new(init_virtio_blk().unwrap()));
    }
}

pub struct VirtioHal;

unsafe impl Hal for VirtioHal {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (VirtioPhysAddr, NonNull<u8>) {
        let mut base = 0;
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            let frame_pa: PhysAddr = frame.ppn.into();
            let frame_pa = frame_pa.into();
            
            // 将分配的页面清零
            let va = frame_pa as *mut u8;
            unsafe {
                core::ptr::write_bytes(va, 0, PAGE_SIZE);
            }
            
            core::mem::forget(frame);
            if i == 0 {
                base = frame_pa;
            }
            // 确保页面是连续的
            assert_eq!(frame_pa, base + i * PAGE_SIZE);
        }
        
        // 记录分配信息
        let base_page = base / PAGE_SIZE;
        info!("virtio_dma_alloc: {:#x} {:}", base_page, pages);
        
        // 确保返回的物理地址按页面大小对齐
        assert_eq!(base % PAGE_SIZE, 0);
        
        // 创建并返回有效的NonNull指针
        let vaddr = unsafe { NonNull::new_unchecked(base as *mut u8) };
        
        (base, vaddr)
    }

    unsafe fn dma_dealloc(paddr: VirtioPhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        // 确保物理地址按页面大小对齐
        assert_eq!(paddr % PAGE_SIZE, 0);
        
        info!("virtio_dma_dealloc: {:#x} {:}", paddr / PAGE_SIZE, pages);
        
        let mut pa = paddr;
        for _i in 0..pages {
            // 转换物理地址为PhysPageNum后释放
            let ppn = PhysAddr::from(pa).into();
            frame_dealloc(ppn);
            
            // 移动到下一个页面
            pa += PAGE_SIZE;
        }
        
        0  // 成功返回0
    }

    unsafe fn mmio_phys_to_virt(paddr: VirtioPhysAddr, _size: usize) -> NonNull<u8> {
        // 在我们的系统中，可能需要通过页表进行映射
        // 这里简化处理，假设物理地址可以直接访问
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> VirtioPhysAddr {
        // 获取缓冲区的起始地址和长度
        let ptr = buffer.as_ptr() as *mut u8;
        
        // 在实际系统中，这里需要查找或创建物理内存映射
        // 简化起见，假设虚拟地址可以直接转换为物理地址
        ptr as usize as VirtioPhysAddr
    }

    unsafe fn unshare(_paddr: VirtioPhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
    }
}

pub struct VirtIOBlock(UPSafeCell<VirtIOBlk<VirtioHal, PciTransport>>);

impl BlockDevice for VirtIOBlock {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0
            .exclusive_access()
            .read_blocks(block_id, buf)
            .expect("Error when reading VirtIOBlk");
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        self.0
            .exclusive_access()
            .write_blocks(block_id, buf)
            .expect("Error when writing VirtIOBlk");
    }
}

impl VirtIOBlock {
    pub fn new(transport: PciTransport) -> Self {
        unsafe {
            Self(UPSafeCell::new(
                VirtIOBlk::<VirtioHal, PciTransport>::new(transport).expect("REASON")
            ))
        }
    }
}



#[allow(unused)]
pub fn block_device_test() {
    let block_device = BLOCK_DEVICE.clone();
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        block_device.read_block(i as usize, &mut read_buffer);
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}


#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Cell<T>(UnsafeCell<T>);

unsafe impl<T> Sync for Cell<T> {}

impl<T> Cell<T> {
    #[inline(always)]
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> Deref for Cell<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> DerefMut for Cell<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get()
    }
}