use crate::*;
use alloc::sync::Arc;

mod ahci;
mod pci;

/// Used only for initialization hacks.
pub const DUMMY_BLOCK_DEVICE: *const dyn BlockDevice = unsafe { transmute(&0 as *const _ as *const ahci::AHCIDriver as *const dyn BlockDevice) };

pub static BLOCK_DEVICE: Cell<Arc<dyn BlockDevice>> = unsafe { transmute(DUMMY_BLOCK_DEVICE) };

pub fn init() {
  unsafe { (BLOCK_DEVICE.get() as *mut Arc<dyn BlockDevice>).write(Arc::new(pci::init().unwrap())); }
}
