mod virtio_blk;

use log::debug;
pub use virtio_blk::VirtIOBlock;
use crate::board::BlockDeviceImpl;
use alloc::sync::Arc;
use easy_fs::BlockDevice;
use lazy_static::*;

lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}

#[allow(unused)]
pub fn block_device_test() {
    debug!("block_device begin!");
    let block_device = BLOCK_DEVICE.clone();
    debug!("block_device ok!");
    let mut write_buffer = [0u8; 512];
    let mut read_buffer = [0u8; 512];
    for i in 0..512 {
        for byte in write_buffer.iter_mut() {
            *byte = i as u8;
        }
        block_device.write_block(i as usize, &write_buffer);
        debug!("write_block ok!");
        block_device.read_block(i as usize, &mut read_buffer);
        debug!("read_block ok!");
        assert_eq!(write_buffer, read_buffer);
    }
    println!("block device test passed!");
}